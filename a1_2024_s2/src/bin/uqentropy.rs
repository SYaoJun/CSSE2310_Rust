use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::exit;

use a1_2024_s2::utils::log::init_logging;
use anyhow::Result;
use log;
use thiserror::Error;

#[derive(Debug, Error)]
enum ExitError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File(String),
}
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

    let result = password.len() as f64 * log2(s as f64);
    floor_to_one_decimal(result)
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
    password
        .chars()
        .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
}
#[warn(clippy::too_many_lines)]
fn read_file(filenames: &[String], passwords: &mut Vec<String>, config: &Config) {
    let mut error_occurred = false;
    // 打印文件数量
    log::info!(
        "Reading {} file{}",
        filenames.len(),
        if filenames.len() == 1 { "" } else { "s" }
    );
    for fname in filenames {
        if let Err(_) = read_single_file(fname, passwords, config) {
            error_occurred = true;
        }
    }
    if error_occurred {
        exit(ExitCodes::InvalidFile as i32);
    }
}

fn read_single_file(
    fname: &String,
    passwords: &mut Vec<String>,
    _config: &Config,
) -> Result<(), ()> {
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
                } else {
                    invalid_lines += 1;
                }
            }
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
            println!(
                "Candidate password would be matched on guess number {}",
                i + 1
            );
            std::io::stdout().flush().unwrap();
            return Some(log2(2.0 * (i + 1) as f64));
        }
    }
    None
}
#[warn(clippy::too_many_lines)]
fn calculate_entropy_two(password: &str, passwords: &[String], config: &Config) -> f64 {
    log::info!("passwords size = {}", passwords.len());
    let mut password_scale = 0;
    if let Some(entropy) = do_basic_match(password, passwords, &mut password_scale) {
        return entropy;
    }

    // --case
    if config.case_sensitive {
        if let Some(entropy) = check_case_match(password, passwords, &mut password_scale) {
            return entropy;
        }
    }

    // --digit-append
    if config.digit_append {
        if let Some(entropy) =
            check_digit_append_match(password, passwords, config, &mut password_scale)
        {
            return entropy;
        }
    }

    // --double-check
    if config.double_check {
        if let Some(entropy) = check_double_match(password, passwords, &mut password_scale) {
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
    println!(
        "No match would be found after checking {} passwords",
        password_scale
    );
    std::io::stdout().flush().unwrap();
    f64::MAX
}

fn check_case_match(password: &str, passwords: &[String], password_scale: &mut i32) -> Option<f64> {
    for (_i, pwd) in passwords.iter().enumerate() {
        let letter_count = get_letter_count(pwd);
        *password_scale += 2_i32.pow(letter_count as u32) - 1;
        if pwd.to_uppercase() == password.to_uppercase() {
            println!(
                "Candidate password would be matched on guess number {}",
                *password_scale
            );
            std::io::stdout().flush().unwrap();
            return Some(log2(2.0 * (*password_scale as f64)));
        }
    }

    None
}
fn dfs(
    password: &str,
    leet_map: &HashMap<char, &str>,
    pwd_chars: &mut [char],
    index: usize,
) -> bool {
    if index == pwd_chars.len() {
        // 打印当前的字符串pwd_chars
        log::info!(
            "transformed pwd: {}, target password: {}",
            pwd_chars.iter().collect::<String>(),
            password
        );
        return pwd_chars.iter().collect::<String>() == password;
    }

    let current_char = pwd_chars[index];

    if let Some(replacements) = leet_map.get(&current_char) {
        for leet_char in replacements.chars() {
            let original = pwd_chars[index];
            pwd_chars[index] = leet_char;

            if dfs(password, leet_map, pwd_chars, index + 1) {
                return true;
            }

            pwd_chars[index] = original;
        }
    }

    // 保持原始字符不变，继续搜索
    if dfs(password, leet_map, pwd_chars, index + 1) {
        return true;
    }
    return false;
}

fn check_leet_match(
    password: &str,
    passwords: &[String],
    _config: &Config,
    password_scale: &mut i32,
) -> Option<f64> {
    // 修复：使用正确的 HashMap 构造方式，移除重复键
    let leet_map: HashMap<char, &str> = HashMap::from([
        ('a', "4@"),
        ('b', "68"),
        ('e', "3"),
        ('g', "69"),
        ('i', "1!"),
        ('l', "1"),
        ('o', "0"),
        ('s', "5$"),
        ('t', "7+"),
        ('x', "%"),
        ('z', "2"),
        // 移除了大写字母的重复映射，因为 chars() 是大小写敏感的
        ('A', "4@"),
        ('B', "68"),
        ('E', "3"),
        ('G', "69"),
        ('I', "1!"),
        ('L', "1"),
        ('O', "0"),
        ('S', "5$"),
        ('T', "7+"),
        ('X', "%"),
        ('Z', "2"),
    ]);

    for pwd in passwords {
        let mut power_one = 0;
        let mut power_two = 0;
        let len = pwd.len();

        // 修复：使用更安全的方式遍历字符
        for c in pwd.chars() {
            if let Some(value) = leet_map.get(&c) {
                match value.len() {
                    1 => power_one += 1,
                    _ => power_two += 1,
                }
            }
        }

        if power_one + power_two == 0 {
            continue;
        }

        // 修复：避免整数溢出，使用 checked_pow
        let scale_increment = 2_i32
            .checked_pow(power_one as u32)
            .and_then(|x| x.checked_mul(3_i32.checked_pow(power_two as u32)?))
            .and_then(|x| x.checked_sub(1))
            .unwrap_or(i32::MAX); // 处理溢出情况

        *password_scale = password_scale.saturating_add(scale_increment);
        log::info!("current scale: {}, pwd: {}", *password_scale, pwd);
        if len != password.len() {
            continue;
        }

        // 将字典密码转换为可变的char数组，以便进行leet变换
        let mut pwd_chars: Vec<char> = pwd.chars().collect();
        log::info!("dict pwd: {}", pwd);

        // 调用dfs函数，将字典密码进行leet变换，然后与目标密码比较
        if dfs(password, &leet_map, &mut pwd_chars, 0) {
            println!(
                "Candidate password would be matched on guess number {}",
                *password_scale
            );
            std::io::stdout().flush().unwrap();
            return Some(log2(2.0 * (*password_scale as f64)));
        }
    }
    None
}

fn check_digit_append_match(
    password: &str,
    passwords: &[String],
    config: &Config,
    password_scale: &mut i32,
) -> Option<f64> {
    let power_table = [10, 100, 1000, 10000, 100000, 1000000, 10000000];

    for (_i, pwd) in passwords.iter().enumerate() {
        let last_char = pwd.chars().last()?;
        if !last_char.is_ascii_digit() {
            for j in 0..config.num_digits {
                for value in 0..power_table[j] {
                    let digit_append = format!("{:0width$}", value, width = j + 1);
                    let new_pwd = format!("{}{}", pwd, digit_append);
                    *password_scale += 1;
                    if new_pwd == password {
                        println!(
                            "Candidate password would be matched on guess number {}",
                            *password_scale
                        );
                        return Some(log2(2.0 * (*password_scale as f64)));
                    }
                }
            }
        }
    }
    None
}

fn check_double_match(
    password: &str,
    passwords: &[String],
    password_scale: &mut i32,
) -> Option<f64> {
    for (_i, first) in passwords.iter().enumerate() {
        let len1 = first.len();
        if len1 > password.len() || !password.starts_with(first) {
            *password_scale += passwords.len() as i32;
            continue;
        }
        for (_j, second) in passwords.iter().enumerate() {
            *password_scale += 1;

            let len2 = second.len();
            if len1 + len2 != password.len() {
                continue;
            }
            let new_pwd = format!("{}{}", first, second);
            if new_pwd == password {
                println!(
                    "Candidate password would be matched on guess number {}",
                    *password_scale
                );
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
    // 把错误传播到最外层，统一处理
    let (config, filenames, file_present) = parse_arguments(&args);

    // 当使用选项时必须提供文件
    if (config.leet || config.case_sensitive || config.digit_append || config.double_check)
        && filenames.is_empty()
    {
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }

    let mut passwords: Vec<String> = Vec::new();
    if !filenames.is_empty() {
        read_file(&filenames, &mut passwords, &config);
    }

    process_user_input(&config, &passwords, file_present);
}

// 这里返回的是三元组
fn parse_arguments(args: &[String]) -> (Config, Vec<String>, bool) {
    let mut filenames = Vec::new();
    let mut config = Config::new();

    let mut i = 1;
    let mut file_present = false;
    while i < args.len() {
        // 这里 match用得妙呀
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
                std::io::stdout().flush().unwrap();
            }
            Err(_) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_entropy() {
        assert_eq!(calculate_entropy("password"), 37.6);
    }

    #[test]
    fn test_calculate_entropy_two() {
        let passwords = vec!["<PASSWORD>".to_string(), "123456".to_string()];
        assert_eq!(
            calculate_entropy_two("password", &passwords, &Config::new()),
            f64::MAX
        );
    }
}
