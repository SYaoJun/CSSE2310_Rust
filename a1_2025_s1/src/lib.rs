use anyhow::{Result, bail};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExitError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File,
}
#[derive(Debug)]
pub enum ExitCodes {
    Usage = 18,
    InvalidFile = 20,
    NoStrong = 14, // 确保NoStrong对应的值为14，与测试期望一致
}

pub const USAGE_MSG: &str = "Usage: ./uqexpr [--init string] [--significantfigures 2..8] [--forloop string] [inputfilename]";

// 是不是可以自己定义错误处理枚举类型
pub struct Config {
    pub init_string: String,
    pub significant_figures: u8,
    pub for_loop: String,
    pub input_filename: String,
}

pub fn handle_command_line_arguments() -> Result<Config, ExitError> {
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config {
        init_string: String::from(""),
        significant_figures: 0,
        for_loop: String::from(""),
        input_filename: String::from(""),
    };
    let mut i = 1;
    let mut init_flag = false;
    let mut figure_flag = false;
    let mut forloop_flag = false;
    let mut filename_flag = false;
    while i < args.len() {
        if args[i].is_empty() {
            return Err(ExitError::Usage);
        }
        match args[i].as_str() {
            "--init" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if init_flag {
                    return Err(ExitError::Usage);
                }
                init_flag = true;
                i += 1;
            }
            "--significantfigures" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if figure_flag {
                    return Err(ExitError::Usage);
                }
                if has_leading_zero(&args[i + 1].as_str()) {
                    return Err(ExitError::Usage);
                }
                figure_flag = true;
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
                i += 1;
            }
            "--forloop" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if forloop_flag {
                    return Err(ExitError::Usage);
                }
                forloop_flag = true;
                i += 1;
            }
            _ => {
                if args[i].starts_with("--") {
                    return Err(ExitError::Usage);
                }
                if filename_flag {
                    return Err(ExitError::Usage);
                }
                filename_flag = true;
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
