use a1_2025_s1::*;
use std::process::exit;

fn main() {
    let res = handle_command_line_arguments();
    if res.is_err() {
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }
    // 如果提前判断了一定不会 panic，可以直接在这里调用 unwrap()函数吗？
    let mut config = res.unwrap();
    if config.filename_flag {
        let file_string_res = check_input_filename(&config.input_filename);
        if file_string_res.is_err() {
            eprintln!(
                "uqexpr: unable to read from input file \"{}\"",
                config.input_filename
            );
            exit(ExitCodes::InvalidFile as i32);
        }
    }
    let res = check_variable(&mut config);
    if res.is_err() {
        eprintln!("{}", VARIABLE_MSG);
        exit(ExitCodes::Variable as i32);
    }
}
