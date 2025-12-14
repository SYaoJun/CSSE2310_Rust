use std::fs::File;
use anyhow::{bail, Result};
use thiserror::Error;
use std::process::exit;
use std::io::{self, BufRead, BufReader, Write};


#[derive(Debug, Error)]
enum ExitError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File,
}
#[derive(Debug)]
enum ExitCodes {
    Usage = 2,
    InvalidFile = 20,
    NoStrong = 14, // 确保NoStrong对应的值为14，与测试期望一致
}

const USAGE_MSG: &str = "Usage: ./uqexpr [--init string] [--significantfigures 2..8] [--forloop string] [inputfilename]";

// 是不是可以自己定义错误处理枚举类型
struct Config{
    init_string: String,
    significant_figures: u8,
    for_loop:String,
    input_filename:String
}


fn handle_command_line_arguments()->Result<Config, ExitError>{
     let args: Vec<String> = std::env::args().collect();
    let config = Config{
        init_string: String::from(""),
        significant_figures: 0,
        for_loop: String::from(""),
        input_filename: String::from(""),
    };
    let mut i = 1;
    let mut init_flag = false;
   while i < args.len(){
       if args[i].eq("--init"){
           if init_flag{
               return Err(ExitError::Usage);
           }
           init_flag = true;
           i += 1;
       }
   }
    Ok(config)
}

fn check_input_filename(filename: &String)->Result<Vec<String>, ExitError>{

    let file = File::open(filename);
    if file.is_err(){
        return Err(ExitError::File);
    }
    let reader = BufReader::new(file.unwrap());
    let mut file_string:Vec<String> = vec![];
    for line_result in reader.lines() {
        match line_result {
            Ok(line)=>{
                file_string.push(line);
            }
            Err(_)=>{
                return Err(ExitError::File);
            }
        }
    }
    Err(ExitError::File)
}
fn main() {


    let res = handle_command_line_arguments();
    if res.is_err(){
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }
    // 如果提前判断了一定不会 panic，可以直接在这里调用 unwrap()函数吗？
    let config = res.unwrap();
    let file_string_res = check_input_filename(&config.input_filename);
    if file_string_res.is_err(){
        eprintln!("uqexpr: unable to read from input file \"{}\"", config.input_filename);
        exit(ExitCodes::Usage as i32);
    }
    let file_string = file_string_res.unwrap();
}




mod tests{
    #[test]
    fn command_line_basic_test(){

    }
}