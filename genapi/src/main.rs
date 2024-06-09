use handlebars::Handlebars;
use rmpv::{decode, Value};
use serde::Serialize;
use std::fs;
use std::process::Command;


/// The name and type of a function/struct parameter
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Parameter {
    name: String,
    parameter_type: String
}


/// The attributes needed to construct a Rust function signature
#[derive(Clone, Debug, Serialize)]
pub struct Function {
    name: String,
    since: u64,
    parameters: Vec<Parameter>,
    return_type: String,
    method: bool
}


impl Function {
    /// Create Function from rmpv::Value
    fn from_value(value: &Value) -> Function {
        let args = match value {
            Value::Map(args) => args,
            _ => panic!()
        };
 
        let mut name = String::new();
        let mut since: u64 = 0;
        let mut parameters: Vec<Parameter> = Vec::new();
        let mut return_type = String::new();
        let mut method = false;
        for (k, v) in args {
            match k {
                x if x.as_str().unwrap() == "name" => {
                    name = v.as_str().unwrap().to_string();
                },
                x if x.as_str().unwrap() == "since" => {
                    since = v.as_u64().unwrap();
                },
                x if x.as_str().unwrap() == "parameters" => {
                    for param in v.as_array().unwrap().iter() {
                        parameters.push(
                            Parameter { name: param[1].as_str().unwrap().to_string(),
                                        parameter_type: value_to_type(&param[0].as_str().unwrap())
                            });
                    }
                },
                x if x.as_str().unwrap() == "return_type" => {
                    return_type = value_to_type(&v.as_str().unwrap())
                },
                x if x.as_str().unwrap() == "method" => {
                    method = v.as_bool().unwrap();
                },
                _ => ()
            }
        }

        Function { name, since, parameters, return_type, method }
    }
}


/// The attributes needed to construct a Rust struct
#[derive(Debug, Serialize)]
pub struct Type {
    name: String,
    parameters: Vec<Parameter>
}


impl Type {
    /// Create Function from rmpv::Value
    pub fn from_map(map: &(Value, Value)) -> Type {
        let (key, value) = map;
        let args = match value {
            Value::Map(args) => args,
            _ => panic!()
        };
 
        let name = key.as_str().unwrap().to_string();
        let mut parameters: Vec<Parameter> = Vec::new();
        for (k, v) in args {
            match k {
                x if x.as_str().unwrap() == "parameters" => {
                    for param in v.as_array().unwrap().iter() {
                        parameters.push(
                            Parameter { name: param[1].as_str().unwrap().to_string(),
                                        parameter_type: value_to_type(&param[0].as_str().unwrap())
                            }
                        );
                    }
                },
                _ => ()
            }
        }

        Type { name, parameters }
    }
}


/// Map MessagePack types to Rust
fn value_to_type(value: &str) -> String {
    match value {
        "Integer" => "i64".to_string(),
        "Boolean" => "bool".to_string(),
        "void" => "".to_string(),
        array if array.starts_with("ArrayOf(") => {
            format!(
                "Vec<{}>",
                value_to_type(
                    array.split_terminator(['(', ')']).collect::<Vec<&str>>()[1])
                )
        },
        other => other.to_string()
    }
}


/// Generate Function structs for each function in the API
fn parse_functions(functions: &Value) -> Vec<Function> {
    let arr = match functions {
        Value::Array(arr) => arr,
        _ => panic!()
    };

    arr.iter()
        .map(|x| Function::from_value(x))
        .collect()
}


/// Generate Rust struct/enums for each type in the API
fn parse_types(types: &Value) -> Vec<Type> {
    let map = match types {
        Value::Map(map) => map,
        _ => panic!()
    };

    map.iter()
        .map(|x| Type::from_map(x))
        .collect()

}


/// Generate Rust files containing the Neovim API
fn generate_api(
    types: Option<Vec<Type>>,
    functions: Option<Vec<Function>>
) {
    let mut reg = Handlebars::new();
    reg.register_template_file("buffer", "genapi/templates/buffer.hbs").unwrap();
    reg.register_template_file("type", "genapi/templates/type.hbs").unwrap();
    reg.register_template_file("function", "genapi/templates/function.hbs").unwrap();

    let mut text = String::new();

    if let Some(types) = types {
        for t in types {
            text = format!("{}{}", text, reg.render("type", &t).unwrap());
        }
    }

    let mut buffer_functions: Vec<Function> = Vec::new();
    let buffer_type = Parameter { name: "buffer".to_string(), parameter_type: "Buffer".to_string() };
    if let Some(functions) = functions {
        for f in functions {
            match &f {
                f if f.name.starts_with("nvim_buf_") => { 
                    let mut fbuf = f.clone();
                    fbuf.name = f.name.strip_prefix("nvim_buf_").unwrap().to_string();
                    let p = fbuf.parameters;
                    fbuf.parameters = p.into_iter().filter(|x| x.name != "buffer".to_string()).collect();
                    buffer_functions.push(fbuf);
                },
                _ => ()
            }
            text = format!("{}{}", text, reg.render("function", &f).unwrap());
        }
    }

    let buf_text = reg.render("buffer", &buffer_functions).unwrap();

    println!("cargo:warning={:?}", buffer_functions);

    fs::create_dir_all("build").expect("Unable to create folder");
    fs::write("build/buffer.rs", buf_text).expect("Unable to write file");
    fs::write("build/functions.rs", text).expect("Unable to write file");
}


fn main() {
    let output = Command::new("nvim")
        .args(["--api-info"])
        .output()
        .expect("Failed to retrieve Neovim API");
    let mut stdout = &output.stdout[..];

    let api = decode::read_value(&mut stdout).unwrap();
    let mut types: Option<Vec<Type>> = None;
    let mut functions: Option<Vec<Function>> = None;
    match api {
        Value::Map(map) => {
            for (k, v) in map.iter() {
                match k {
                    x if x.as_str().unwrap() == "version" => {
                        println!("cargo:warning={}", x);
                    },
                    x if x.as_str().unwrap() == "functions" => {
                        functions = Some(parse_functions(v));
                    },
                    x if x.as_str().unwrap() == "types" => {
                        types = Some(parse_types(v));
                    }
                    other => println!("cargo:warning=Other: {}", other)
                }
            }
        },
        _ => ()
    }

    generate_api(types, functions);
}
