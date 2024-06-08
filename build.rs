use handlebars::Handlebars;
use rmpv::{decode, Value};
use serde::Serialize;
use std::fs;
use std::process::Command;


#[derive(Debug, Serialize)]
pub struct Parameter {
    name: String,
    parameter_type: String
}


#[derive(Debug, Serialize)]
pub struct Function {
    name: String,
    since: u64,
    parameters: Vec<Parameter>,
    return_type: String,
    method: bool
}

impl Function {
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


fn parse_functions(functions: &Value) -> Vec<Function> {
    let arr = match functions {
        Value::Array(arr) => arr,
        _ => panic!()
    };

    arr.iter()
        .map(|x| Function::from_value(x))
        .collect()
}


fn generate_api(
    functions: Option<Vec<Function>>
) {
    let mut reg = Handlebars::new();
    reg.register_template_file("function", "templates/function.hbs").unwrap();

    let mut text = String::new();
    if let Some(functions) = functions {
        for func in functions {
            text = format!("{}{}", text, reg.render("function", &func).unwrap());
        }
    }

    fs::write("templates/functions.rs", text).expect("Unable to write file");
}


fn main() {
    let output = Command::new("nvim")
        .args(["--api-info"])
        .output()
        .expect("Failed to retrieve Neovim API");
    let mut stdout = &output.stdout[..];

    let api = decode::read_value(&mut stdout).unwrap();
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
                    other => println!("cargo:warning=Other: {}", other)
                }
            }
        },
        _ => ()
    }

    generate_api(functions);
}
