use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::exit;

static mut LEET: bool = false;
static mut CASE_SENSITIVE: bool = false;
static mut DIGIT_APPEND: bool = false;
static mut DOUBLE_CHECK: bool = false;
static mut NUM_DIGITS: usize = 0;
static mut PASSWORD_COUNT: usize = 0;

// 定义字符串常量
const USAGE_MSG: &str =
    "Usage: ./uqentropy [--leet] [--double] [--digit-append 1..8] [--case] [listfilename ...]";

#[derive(Debug)]
enum ExitCodes {
    Usage = 2,
    InvalidFile = 20,
    NoStrong = 14,
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

fn read_file(filenames: &[String], passwords: &mut Vec<String>) {
    let mut found_error = false;

    for fname in filenames {
        let file = File::open(fname);
        if file.is_err() {
            eprintln!("uqentropy: unable to open file \"{}\" for reading", fname);
            found_error = true;
            continue;
        }
        let file = file.unwrap();
        let reader = BufReader::new(file);
        let mut local_count = 0;
        for line in reader.lines().map_while(Result::ok) {
            // 检查line 是否只包含 空格 和 可打印的字符
            if !check_password_is_valid(&line) {
                eprintln!("uqentropy: invalid character found in file \"{}\"", fname);
                found_error = true;
                break;
            }

            for token in line.split_whitespace() {
                if !token.is_empty() {
                    passwords.push(token.to_string());
                    local_count += 1;
                    unsafe {
                        PASSWORD_COUNT += 1;
                    }
                }
            }
        }
        if found_error {
            continue;
        }
        if local_count == 0 {
            eprintln!("uqentropy: \"{}\" does not contain any passwords", fname);
            std::io::stderr().flush().unwrap();
            found_error = true;
        }
    }

    if found_error {
        exit(ExitCodes::InvalidFile as i32);
    }
}

fn check_password_is_valid(password: &str) -> bool {
    !password.is_empty()
        && password
            .chars()
            .all(|c| c.is_ascii_graphic() && !c.is_whitespace())
}
fn floor_to_one_decimal(x: f64) -> f64 {
    (x * 10.0).floor() / 10.0
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut filenames = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--leet" => unsafe { LEET = true },
            "--case" => unsafe { CASE_SENSITIVE = true },
            "--double" => unsafe { DOUBLE_CHECK = true },
            "--digit-append" => {
                if i + 1 >= args.len() {
                    eprintln!("{}", USAGE_MSG);
                    exit(ExitCodes::Usage as i32);
                }
                unsafe {
                    NUM_DIGITS = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("{}", USAGE_MSG);
                        exit(ExitCodes::Usage as i32);
                    });
                    if NUM_DIGITS < 1 || NUM_DIGITS > 8 {
                        eprintln!("{}", USAGE_MSG);
                        exit(ExitCodes::Usage as i32);
                    }
                    DIGIT_APPEND = true;
                }
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
                break;
            }
        }
        i += 1;
    }

    if (unsafe { LEET || CASE_SENSITIVE || DIGIT_APPEND || DOUBLE_CHECK }) && filenames.is_empty() {
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }

    let mut passwords: Vec<String> = Vec::new();
    read_file(&filenames, &mut passwords);

    println!("Welcome to UQEntropy!");
    println!("Written by s4905773.");
    println!("Enter candidate passwords to check their strength.");
    // flush
    std::io::stdout().flush().unwrap();
    // 遍历每个filenames 并打开文件
    let stdin = io::stdin();
    let mut count_strong = 0;
    // 会在 EOF 时自动退出循环
    for line in stdin.lock().lines() {
        let mut password = line.unwrap();
        password = password.trim_end().to_string();

        if !check_password_is_valid(&password) {
            eprintln!("Password is invalid");
            continue;
        }
        let entropy = calculate_entropy(&password);
        if entropy >= 60.0 {
            count_strong += 1;
        }
        println!(
            "Password entropy calculated to be {:.1}",
            floor_to_one_decimal(entropy)
        );
        println!("Password strength rating: {}", map_to_strength(entropy));
    }
    if count_strong == 0 {
        println!("No strong password(s) have been identified");
        exit(ExitCodes::NoStrong as i32)
    } else {
        exit(0);
    }
}
