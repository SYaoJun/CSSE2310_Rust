use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::exit;
use std::fs;
use log;
use env_logger;

struct Config {
    leet: bool,
    case_sensitive: bool,
    digit_append: bool,
    double_check: bool,
    num_digits: usize,
}

impl Config {
    fn new() -> Self {
        Config {
            leet: false,
            case_sensitive: false,
            digit_append: false,
            double_check: false,
            num_digits: 0,
        }
    }
}

// 定义字符串常量
const USAGE_MSG: &str =
    "Usage: ./uqentropy [--leet] [--double] [--digit-append 1..8] [--case] [listfilename ...]";

#[derive(Debug)]
enum ExitCodes {
    Usage = 2,
    InvalidFile = 20,
    NoStrong = 14, // 确保NoStrong对应的值为14，与测试期望一致
}

fn log2(x: f64) -> f64 {
    x.log2()
}

/// 初始化日志系统，创建带时间戳的日志文件
fn init_logging() {
    // 创建log目录（如果不存在）
    fs::create_dir_all("log").expect("无法创建log目录");
    
    let the_time = chrono::Local::now()
        .format("%Y_%m_%d_%H:%M:%S")
        .to_string();


    // 构造日志文件路径
    let log_file_path = format!("log/uqentropy_{}.log", the_time);
    
    // 设置环境变量以配置env_logger
    std::env::set_var("RUST_LOG", "debug");
    
    // 初始化env_logger并设置输出到文件
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)
        .expect("无法创建日志文件");
    
    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .format_timestamp_secs()
        .init();
    
    log::info!("日志系统已初始化，日志文件: {}", log_file_path);
}

fn calculate_entropy(password: &str) -> f64 {
    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    let mut has_symbol = false;

    for c in password.chars() {
        if c.is_ascii_lowercase() {
            has_lower = true;
        } else if c.is_ascii_uppercase() {
            has_upper = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else {
            has_symbol = true;
        }
    }

    let mut s = 0;
    if has_digit {
        s += 10;
    }
    if has_lower {
        s += 26;
    }
    if has_upper {
        s += 26;
    }
    if has_symbol {
        s += 32;
    }

    password.len() as f64 * log2(s as f64)
}

fn map_to_strength(entropy: f64) -> &'static str {
    if entropy < 35.0 {
        "very weak"
    } else if entropy < 60.0 {
        "weak"
    } else if entropy < 120.0 {
        "strong"
    } else {
        "very strong"
    }
}

fn check_password_is_valid(password: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    // 只允许可打印的ASCII字符，不包括控制字符
    password.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
}
#[warn(clippy::too_many_lines)]
fn read_file(filenames: &[String], passwords: &mut Vec<String>, config: &Config) {
    let mut error_occured = false;
    // 打印文件数量
    log::info!("Reading {} file{}", filenames.len(), if filenames.len() == 1 {""} else {"s"});
    for fname in filenames {
        if let Err(_) = read_single_file(fname, passwords, config) {
            error_occured = true;
        }
    }
    if error_occured {
        exit(ExitCodes::InvalidFile as i32);
    }
}

fn read_single_file(fname: &String, passwords: &mut Vec<String>, _config: &Config) -> Result<(), ()> {
    let file = File::open(fname).map_err(|_| {
        eprintln!("uqentropy: unable to open file \"{}\" for reading", fname);
    })?;
    
    let reader = BufReader::new(file);
    let mut has_valid_password = false;
    let mut invalid_lines = 0;
    let mut total_lines = 0;
    for line_result in reader.lines() {
        total_lines += 1;
        match line_result {
            Ok(line) => {
                if process_line(&line.trim(), passwords, fname) {
                    has_valid_password = true;
                }else{
                    invalid_lines += 1;
                }
            },
            Err(_) => {
                eprintln!("uqentropy: error reading file \"{}\"", fname);
                return Err(());
            }
        }
    }
    log::info!("total lines: {}", total_lines);
    if !has_valid_password {
        eprintln!("uqentropy: \"{}\" does not contain any passwords", fname);
        std::io::stderr().flush().unwrap();
        return Err(());
    }
    if invalid_lines > 0 {
        return Err(());
    }
    Ok(())
}

fn process_line(line: &str, passwords: &mut Vec<String>, fname: &str) -> bool {
    // 检查行中是否包含无效字符（除了ASCII空白字符）
    for c in line.chars() {
        if !c.is_ascii() || (c.is_ascii() && !c.is_ascii_graphic() && !c.is_ascii_whitespace()) {
            log::debug!("Invalid character '{}' found in line: {}", c, line);
            eprintln!("uqentropy: invalid character found in file \"{}\"", fname);
            return false;
        }
    }
    
    let mut has_valid_password = false;
    // 分割行并添加非空密码
    for token in line.split_whitespace() {
        if !token.is_empty() && check_password_is_valid(token) {
            passwords.push(token.to_string());
            has_valid_password = true;
        } else if !token.is_empty() {
            log::info!("Filtered out token: '{}' from line: '{}'", token, line);
        }
    }
    has_valid_password
}

fn floor_to_one_decimal(x: f64) -> f64 {
    (x * 10.0).floor() / 10.0
}
fn leet_transform(password: &str) -> String {
    let mut result = String::new();
    for c in password.chars() {
        match c {
            'a' | 'A' => result.push('4'),
            'e' | 'E' => result.push('3'),
            'i' | 'I' => result.push('1'),
            'o' | 'O' => result.push('0'),
            's' | 'S' => result.push('5'),
            't' | 'T' => result.push('7'),
            'b' | 'B' => result.push('8'),
            'g' | 'G' => result.push('9'),
            _ => result.push(c),
        }
    }
    result
}

/// 计算字符串中字母的数量
fn get_letter_count(s: &str) -> i32 {
    let mut count = 0;
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            count += 1;
        }
    }
    count
}

fn do_basic_match(password: &str, passwords: &[String], password_scale: &mut i32) -> Option<f64> { 
    for (i, pwd) in passwords.iter().enumerate() {
        *password_scale += 1;
        if pwd == password {
            println!("Candidate password would be matched on guess number {}", i + 1);
            std::io::stdout().flush().unwrap();
            return Some(log2(2.0 * (i + 1) as f64));
        }
    }
    None
}
#[warn(clippy::too_many_lines)]
fn calculate_entropy_two(password: &str, passwords: &[String], config: &Config) -> f64 {
    // 遍历所有密码
    log::info!("passwords size = {}", passwords.len());
    let mut password_scale = 0;
    if let Some(entropy) = do_basic_match(password, passwords, &mut password_scale) {
        return entropy;
    }
    
    // --case
    if config.case_sensitive {
        if let Some(entropy) = check_case_match(password, passwords,  &mut password_scale) {
            return entropy;
        }
    }
    
    
    // --digit-append
    if config.digit_append {
        if let Some(entropy) = check_digit_append_match(password, passwords, config, &mut password_scale) {
            return entropy;
        }
    }
    
    // --double-check
    if config.double_check {
        if let Some(entropy) = check_double_match(password, passwords,  &mut password_scale) {
            return entropy;
        }
    }

    // --leet
    if config.leet {
        if let Some(entropy) = check_leet_match(password, passwords, config, &mut password_scale) {
            return entropy;
        }
    }
    
    // If no match found, return a large value
    println!("No match would be found after checking {} passwords", password_scale);
    std::io::stdout().flush().unwrap();
    f64::MAX
}


fn check_case_match(password: &str, passwords: &[String],  password_scale: &mut i32) -> Option<f64> {
    
        for (_i, pwd) in passwords.iter().enumerate() {
            let letter_count = get_letter_count(pwd);
            *password_scale += 2_i32.pow(letter_count as u32) - 1;
            if pwd == password {
                println!("Candidate password would be matched on guess number {}", *password_scale);
                std::io::stdout().flush().unwrap();
                return Some(log2(2.0 * (*password_scale as f64)));
            }
        }
    
    None
}

fn check_leet_match(password: &str, passwords: &[String], config: &Config, password_scale: &mut i32) -> Option<f64> {
    for (_i, pwd) in passwords.iter().enumerate() {
        // 计算Leet变换的数量
        // 这里我们简化处理，实际应该计算所有可能的Leet变换组合
        let letter_count = get_letter_count(pwd);
        let leet_combinations = 2_i32.pow(letter_count as u32) - 1;
        *password_scale += leet_combinations;
        
        // 检查密码是否是基础密码的Leet转换
        let leet_pwd = leet_transform(pwd);
        if leet_pwd == password {
            println!("Candidate password would be matched on guess number {}", *password_scale);
            std::io::stdout().flush().unwrap();
            return Some(log2(2.0 * (*password_scale as f64)));
        }
        
        // 检查基础密码是否是密码的Leet转换
        let leet_input = leet_transform(password);
        if pwd == &leet_input || (!config.case_sensitive && pwd.to_lowercase() == leet_input.to_lowercase()) {
            println!("Candidate password would be matched on guess number {}", *password_scale);
            std::io::stdout().flush().unwrap();
            return Some(log2(2.0 * (*password_scale as f64)));
        }
    }
    None
}

fn check_digit_append_match(password: &str, passwords: &[String], config: &Config, password_scale: &mut i32) -> Option<f64> {
    let power_table = [10, 100, 1000, 10000, 100000, 1000000, 10000000];
    
    for (_i, pwd) in passwords.iter().enumerate() {
       let  last_char = pwd.chars().last()?;
       if !last_char.is_ascii_digit() {
           for j in 0..config.num_digits {
             for value in 0..power_table[j] {
                 let digit_append = format!("{:0width$}", value, width=j+1);
                 let new_pwd = format!("{}{}", pwd, digit_append);
                 *password_scale += 1;
                 if new_pwd == password {
                     println!("Candidate password would be matched on guess number {}", *password_scale);
                     return Some(log2(2.0 * (*password_scale as f64)));
                 }
               }
           }
       }
    }
    None
}

fn check_double_match(password: &str, passwords: &[String],  password_scale: &mut i32) -> Option<f64> {
    for (_i, first) in passwords.iter().enumerate() {
         let len1 = first.len();
         if len1 > password.len() || !password.starts_with(first) {
            *password_scale += passwords.len() as i32;
            continue;
         }
       for (_j, second) in passwords.iter().enumerate(){
             *password_scale += 1;
            
             let len2 = second.len();
             if len1 + len2 != password.len() {
                 continue;
             }
             let new_pwd = format!("{}{}", first, second);
             if new_pwd == password {
                 println!("Candidate password would be matched on guess number {}", *password_scale);
                 return Some(log2(2.0 * (*password_scale as f64)));
             }
         
       }
    }
    None
}
fn main() {
    // 初始化日志系统
    init_logging();
    
    let args: Vec<String> = env::args().collect();
    let (config, filenames, file_present) = parse_arguments(&args);
    
    // 当使用选项时必须提供文件
    if (config.leet || config.case_sensitive || config.digit_append || config.double_check) && filenames.is_empty() {
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }

    let mut passwords: Vec<String> = Vec::new();
    if !filenames.is_empty() {
        read_file(&filenames, &mut passwords, &config);
    }

    process_user_input(&config, &passwords, file_present);
}

fn parse_arguments(args: &[String]) -> (Config, Vec<String>, bool) {
    let mut filenames = Vec::new();
    let mut config = Config::new();

    let mut i = 1;
    let mut file_present = false;
    while i < args.len() {
        match args[i].as_str() {
            "--leet" => config.leet = true,
            "--case" => config.case_sensitive = true,
            "--double" => config.double_check = true,
            "--digit-append" => {
                if i + 1 >= args.len() {
                    eprintln!("{}", USAGE_MSG);
                    exit(ExitCodes::Usage as i32);
                }
                config.num_digits = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("{}", USAGE_MSG);
                    exit(ExitCodes::Usage as i32);
                });
                if config.num_digits < 1 || config.num_digits > 8 {
                    eprintln!("{}", USAGE_MSG);
                    exit(ExitCodes::Usage as i32);
                }
                config.digit_append = true;
                i += 1;
            }
            _ => {
                // if args is empty should exit
                if args[i].is_empty() {
                    eprintln!("{}", USAGE_MSG);
                    exit(ExitCodes::Usage as i32);
                }
                if args[i].starts_with('-') {
                    eprintln!("{}", USAGE_MSG);
                    exit(ExitCodes::Usage as i32);
                }
                for arg in args.iter().skip(i) {
                    filenames.push(arg.clone());
                }
                file_present = true;
                break;
            }
        }
        i += 1;
    }
    (config, filenames, file_present)
}

fn process_user_input(config: &Config, passwords: &[String], file_present: bool) {
    println!("Welcome to UQEntropy!");
    println!("Written by @yaojun.");
    println!("Enter candidate passwords to check their strength.");
    // flush
    std::io::stdout().flush().unwrap();
    
    let stdin = io::stdin();
    let mut count_strong = 0;
    
    // 会在 EOF 时自动退出循环
    for line_result in stdin.lock().lines() {
        match line_result {
            Ok(line) => {
                let password = line.trim_end();

                if !check_password_is_valid(password) {
                    eprintln!("Password is invalid");
                    continue;
                }
                
                let mut entropy = calculate_entropy(password);
                if file_present {
                    let entropy_two = calculate_entropy_two(password, passwords, config);
                    if entropy_two < entropy {
                        entropy = entropy_two;
                    }
                }
                if entropy >= 60.0 {
                    count_strong += 1;
                }
                println!(
                    "Password entropy calculated to be {:.1}",
                    floor_to_one_decimal(entropy)
                );
                println!("Password strength rating: {}", map_to_strength(entropy));
                // 刷新输出
                std::io::stdout().flush().unwrap();
            },
            Err(_) => {
                // 处理输入错误
                break;
            }
        }
    }
    if count_strong == 0 {
        println!("No strong password(s) have been identified");
        std::io::stdout().flush().unwrap();
        exit(ExitCodes::NoStrong as i32)
    } else {
        exit(0);
    }
}
