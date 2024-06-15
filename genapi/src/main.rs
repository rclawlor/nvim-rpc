use handlebars::{Handlebars, handlebars_helper};
use rmpv::{decode, Value};
use regex::Regex;
use serde::Serialize;
use serde_json::Value as SValue;
use std::fs;
use std::process::Command;

mod error;
use error::Error;

/// The name of a struct to impl for and all associated functions
#[derive(Clone, Debug, Serialize)]
pub struct Impl<'a> {
    name: &'a str,
    prefix: &'a str,
    functions: &'a Vec<Function>,
}

/// The name and type of a function/struct parameter
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Parameter {
    name: String,
    parameter_type: String,
}

/// The attributes needed to construct a Rust function signature
#[derive(Clone, Debug, Serialize)]
pub struct Function {
    name: String,
    since: Option<u64>,
    deprecated_since: Option<u64>,
    parameters: Vec<Parameter>,
    return_type: String,
    method: bool,
}

impl Function {
    /// Create Function from rmpv::Value
    fn from_value(value: &Value) -> Function {
        let args = match value {
            Value::Map(args) => args,
            _ => panic!(),
        };

        let mut name = String::new();
        let mut since: Option<u64> = None;
        let mut deprecated_since: Option<u64> = None;
        let mut parameters: Vec<Parameter> = Vec::new();
        let mut return_type = "()".to_string();
        let mut method = false;
        for (k, v) in args {
            match k {
                x if x.as_str().unwrap() == "name" => {
                    name = v.as_str().unwrap().to_string();
                }
                x if x.as_str().unwrap() == "since" => {
                    since = Some(v.as_u64().unwrap());
                }
                x if x.as_str().unwrap() == "deprecated_since" => {
                    deprecated_since = Some(v.as_u64().unwrap());
                }
                x if x.as_str().unwrap() == "parameters" => {
                    for param in v.as_array().unwrap().iter() {
                        parameters.push(Parameter {
                            name: param[1].as_str().unwrap().to_string(),
                            parameter_type: value_to_type(param[0].as_str().unwrap()),
                        });
                    }
                }
                x if x.as_str().unwrap() == "return_type" => {
                    return_type = value_to_type(v.as_str().unwrap())
                }
                x if x.as_str().unwrap() == "method" => {
                    method = v.as_bool().unwrap();
                }
                _ => (),
            }
        }

        Function {
            name,
            since,
            deprecated_since,
            parameters,
            return_type,
            method,
        }
    }
}

/// The attributes needed to construct a Rust struct
#[derive(Debug, Serialize)]
pub struct Type {
    name: String,
    parameters: Vec<Parameter>,
}

impl Type {
    /// Create Function from rmpv::Value
    pub fn from_map(map: &(Value, Value)) -> Type {
        let (key, value) = map;
        let args = match value {
            Value::Map(args) => args,
            _ => panic!(),
        };

        let name = key.as_str().unwrap().to_string();
        let mut parameters: Vec<Parameter> = Vec::new();
        for (k, v) in args {
            match k {
                x if x.as_str().unwrap() == "parameters" => {
                    for param in v.as_array().unwrap().iter() {
                        parameters.push(Parameter {
                            name: param[1].as_str().unwrap().to_string(),
                            parameter_type: value_to_type(param[0].as_str().unwrap()),
                        });
                    }
                }
                _ => (),
            }
        }

        Type { name, parameters }
    }
}

// Check if function returns ()
handlebars_helper!(no_ret: |a: SValue| a != SValue::from("()"));

/// Map MessagePack types to Rust
fn value_to_type(value: &str) -> String {
    match value {
        "Integer" => "i64".to_string(),
        "Float" => "f64".to_string(),
        "Boolean" => "bool".to_string(),
        "void" => "()".to_string(),
        "Array" => "Vec<Value>".to_string(),
        "Object" => "Value".to_string(),
        "LuaRef" => "Value".to_string(),
        array if array.starts_with("ArrayOf(") => {
            let inner = array.split_terminator(['(', ')']).collect::<Vec<&str>>()[1];
            let re = Regex::new(r"([a-zA-Z]+), ([0-9]+)").expect("This is a valid regex");
            if let Some(x) = re.captures(inner) {
                let t = value_to_type(x.get(1).unwrap().as_str());
                let n = x.get(2).unwrap().as_str().parse::<usize>().unwrap(); 
                format!("({})", vec![t; usize::from(n)].join(", "))
            } else {
                format!(
                    "Vec<{}>",
                    value_to_type(array.split_terminator(['(', ')']).collect::<Vec<&str>>()[1])
                )
            }
        },
        "Dictionary" => "Vec<(Value, Value)>".to_string(),
        other => other.to_string(),
    }
}

/// Generate Function structs for each function in the API
fn parse_functions(functions: &Value) -> Vec<Function> {
    let arr = match functions {
        Value::Array(arr) => arr,
        _ => panic!(),
    };

    arr.iter().map(Function::from_value).collect()
}

/// Generate Rust struct/enums for each type in the API
fn parse_types(types: &Value) -> Vec<Type> {
    let map = match types {
        Value::Map(map) => map,
        _ => panic!(),
    };

    map.iter().map(Type::from_map).collect()
}

/// Strips the `prefix` off a function name and removes the `param` to allow for
/// use in an impl.
///
/// # Example
/// ```
/// pub fn nvim_buf_set_mark(buffer: Buffer, name: String) { ... }
///
/// /// The above function signature is modified to become:
/// impl Buffer {
///     pub fn set_mark(&self, name: String) { ... }
/// }
/// ```
fn strip_prefix(f: &Function, prefix: &str, param: &str) -> Function {
    let mut f_mod = f.clone();
    f_mod.name = match f.name.strip_prefix(prefix) {
        Some(name) => name.to_string(),
        None => f_mod.name,
    };
    let p = f_mod.parameters;
    f_mod.parameters = p.into_iter().filter(|x| x.name != *param).collect();

    f_mod
}

/// Some functions in the Neovim API use `fn` as an input name which is a
/// Rust keyword. This function maps these to `function` so that they can be used.
fn change_keywords(f: &Function) -> Function {
    let mut f_mod = f.clone();
    let p = f_mod.parameters;
    f_mod.parameters = p
        .into_iter()
        .map(|x| match x {
            x if x.name == "fn" => {
                let mut x_mod = x.clone();
                x_mod.name = "r#fn".to_string();
                x_mod
            },
            x if x.name == "type" => {
                let mut x_mod = x.clone();
                x_mod.name = "r#type".to_string();
                x_mod
            },
            other => other,
        })
        .collect();

    f_mod
}

/// Save the generated functions to a Rust file
fn save_functions(
    registry: &Handlebars,
    template: &str,
    filename: &str,
    structname: &str,
    prefix: &str,
    param: &str,
    functions: &[Function],
) -> Result<(), Error> {
    fs::write(
        format!("build/{}.rs", filename),
        registry.render(
            template,
            &Impl {
                name: structname,
                prefix,
                functions: &functions
                    .iter()
                    .map(|x| strip_prefix(x, prefix, param))
                    .map(|x| change_keywords(&x))
                    .collect(),
            },
        )?,
    )?;

    Command::new("rustfmt")
        .arg(format!("build/{}.rs", filename))
        .output()
        .expect("Failed to format generated file");

    Ok(())
}

/// Generate Rust files containing the Neovim API
fn generate_api(functions: Option<Vec<Function>>) -> Result<(), Error> {
    let mut registry = Handlebars::new();
    registry
        .register_template_file("nvim", "genapi/templates/nvim.hbs")
        .unwrap();
    registry
        .register_template_file("object", "genapi/templates/object.hbs")
        .unwrap();
    registry
        .register_helper("no_ret", Box::new(no_ret));

    let mut buffer_functions: Vec<Function> = Vec::new();
    let mut nvim_functions: Vec<Function> = Vec::new();
    let mut tabpage_functions: Vec<Function> = Vec::new();
    let mut window_functions: Vec<Function> = Vec::new();
    if let Some(functions) = functions {
        for f in functions {
            if f.deprecated_since.is_none() {
                match &f {
                    f if f.name.starts_with("nvim_buf_") => {
                        buffer_functions.push(f.clone());
                    }
                    f if f.name.starts_with("nvim_tabpage_") => {
                        tabpage_functions.push(f.clone());
                    }
                    f if f.name.starts_with("nvim_win_") => {
                        window_functions.push(f.clone());
                    }
                    f => {
                        nvim_functions.push(f.clone());
                    }
                }
            }
        }
    }

    fs::create_dir_all("build").expect("Unable to create folder");

    save_functions(
        &registry,
        "object",
        "buffer",
        "Buffer",
        "nvim_buf_",
        "buffer",
        &buffer_functions,
    )?;
    save_functions(
        &registry,
        "nvim",
        "nvim",
        "Nvim",
        "nvim_",
        "",
        &nvim_functions
    )?;
    save_functions(
        &registry,
        "object",
        "tabpage",
        "Tabpage",
        "nvim_tabpage_",
        "tabpage",
        &tabpage_functions,
    )?;
    save_functions(
        &registry,
        "object",
        "window",
        "Window",
        "nvim_win_",
        "window",
        &window_functions,
    )?;

    Ok(())
}

fn main() {
    let output = Command::new("nvim")
        .args(["--api-info"])
        .output()
        .expect("Failed to retrieve Neovim API");
    let mut stdout = &output.stdout[..];

    let api = decode::read_value(&mut stdout).unwrap();

    let mut _types: Option<Vec<Type>> = None;
    let mut functions: Option<Vec<Function>> = None;
    if let Value::Map(map) = api {
        for (k, v) in map.iter() {
            match k {
                x if x.as_str().unwrap() == "version" => {
                    println!("cargo:warning={}", x);
                }
                x if x.as_str().unwrap() == "functions" => {
                    functions = Some(parse_functions(v));
                }
                x if x.as_str().unwrap() == "types" => {
                    _types = Some(parse_types(v));
                }
                other => println!("cargo:warning=Other: {}", other),
            }
        }
    }

    generate_api(functions).unwrap();
}
