use rmpv::{decode, Value};
use serde_json;
use std::process::Command;


#[derive(Debug)]
struct Function {
    name: String,
    since: u64,
    parameters: Vec<Value>,
    return_type: Value,
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
        let mut parameters: Vec<Value> = Vec::new();
        let mut return_type = Value::Nil;
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
                    parameters = v.as_array().unwrap().to_vec();
                },
                x if x.as_str().unwrap() == "return_type" => {
                    return_type = v.clone();
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


fn parse_functions(functions: &Value) {
    let arr = match functions {
        Value::Array(arr) => arr,
        _ => panic!()
    };
    
    for function in arr {
        let f = Function::from_value(function);
        println!("cargo:warning={:?}", f);
    }
}


fn main() {
    let output = Command::new("nvim")
        .args(["--api-info"])
        .output()
        .expect("Failed to retrieve Neovim API");
    let mut stdout = &output.stdout[..];

    let api = decode::read_value(&mut stdout).unwrap();
    match api {
        Value::Map(map) => {
            for (k, v) in map.iter() {
                match k {
                    x if x.as_str().unwrap() == "version" => {
                        println!("cargo:warning={}", x);
                    },
                    x if x.as_str().unwrap() == "functions" => {
                        parse_functions(v)
                    },
                    other => println!("cargo:warning=Other: {}", other)
                }
            }
        },
        _ => ()
    }
}
