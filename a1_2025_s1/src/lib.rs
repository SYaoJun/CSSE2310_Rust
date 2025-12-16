use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExitError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File,
    #[error("variable")]
    Variable,
}
#[derive(Debug)]
pub enum ExitCodes {
    Usage = 18,
    InvalidFile = 5,
    Variable = 14,
}

pub const USAGE_MSG: &str = "Usage: ./uqexpr [--init string] [--significantfigures 2..8] [--forloop string] [inputfilename]";
pub const VARIABLE_MSG: &str = "uqexpr: invalid variable(s) were specified";
// 是不是可以自己定义错误处理枚举类型
pub struct Config {
    pub init_string: String,
    pub significant_figures: u8,
    pub for_loop: String,
    pub input_filename: String,
    pub init_flag: bool,
    pub figure_flag: bool,
    pub forloop_flag: bool,
    pub filename_flag: bool,
    pub init_map: std::collections::HashMap<String, f32>,
}

pub fn handle_command_line_arguments() -> Result<Config, ExitError> {
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config {
        init_string: String::from(""),
        significant_figures: 0,
        for_loop: String::from(""),
        input_filename: String::from(""),
        init_flag: false,
        figure_flag: false,
        forloop_flag: false,
        filename_flag: false,
        init_map: std::collections::HashMap::new(),
    };
    let mut i = 1;

    while i < args.len() {
        if args[i].is_empty() {
            return Err(ExitError::Usage);
        }
        match args[i].as_str() {
            "--init" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if config.init_flag {
                    return Err(ExitError::Usage);
                }
                config.init_flag = true;
                i += 2;
            }
            "--significantfigures" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if config.figure_flag {
                    return Err(ExitError::Usage);
                }
                if has_leading_zero(args[i + 1].as_str()) {
                    return Err(ExitError::Usage);
                }
                config.figure_flag = true;
                let parse_int = args[i + 1].parse::<u8>();
                match parse_int {
                    Err(_) => {
                        return Err(ExitError::Usage);
                    }
                    Ok(x) => {
                        if x >= 2 && x <= 8 {
                            config.significant_figures = x;
                        } else {
                            return Err(ExitError::Usage);
                        }
                    }
                }
                i += 2;
            }
            "--forloop" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if config.forloop_flag {
                    return Err(ExitError::Usage);
                }
                config.forloop_flag = true;
                i += 2;
            }
            _ => {
                if args[i].starts_with("--") {
                    return Err(ExitError::Usage);
                }
                if config.filename_flag {
                    return Err(ExitError::Usage);
                }
                config.input_filename = args[i].clone();
                config.filename_flag = true;
                i += 1;
            }
        }
    }
    Ok(config)
}

pub fn has_leading_zero(s: &str) -> bool {
    if s.len() <= 1 {
        return false;
    }
    // Rust中字符串访问每个元素，需要转换为 Vec之后访问
    let chars: Vec<char> = s.chars().collect();

    if chars[0] == '0' {
        return true;
    }
    false
}

pub fn check_input_filename(filename: &String) -> Result<Vec<String>, ExitError> {
    let file = File::open(filename);
    if file.is_err() {
        return Err(ExitError::File);
    }
    let reader = BufReader::new(file.unwrap());
    let mut file_string: Vec<String> = vec![];
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                file_string.push(line);
            }
            Err(_) => {
                return Err(ExitError::File);
            }
        }
    }
    Err(ExitError::File)
}

pub fn check_variable(config: &mut Config) -> Result<(), ExitError> {
    let mut value = 0f32;
    if config.init_flag {
        if !config.init_string.contains("=") {
            return Err(ExitError::Variable);
        }
        let split: Vec<&str> = config.init_string.split("=").collect();
        if split.len() != 2 {
            return Err(ExitError::Variable);
        }
        let var_name = split[0].trim();
        if var_name.is_empty() {
            return Err(ExitError::Variable);
        }
        let var_value = split[1].trim();
        if var_value.is_empty() {
            return Err(ExitError::Variable);
        }
        if !var_value
            .chars()
            .all(|c| c.is_digit(10) || c == '.' || c == '-')
        {
            return Err(ExitError::Variable);
        }
        value = var_value.parse::<f32>().unwrap();
        config.init_map.insert(var_name.to_string(), value);
    }
    Ok(())
}
