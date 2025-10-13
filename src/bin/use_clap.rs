use clap::Parser;
use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::exit;

// -------------------------------
// 常量和全局状态
// -------------------------------
const MAXSIZE: usize = 128;
const MAX_NUM: usize = 128;

static mut LEET: bool = false;
static mut CASE_SENSITIVE: bool = false;
static mut DIGIT_APPEND: bool = false;
static mut DOUBLE_CHECK: bool = false;
static mut NUM_DIGITS: usize = 0;
static mut PASSWORD_COUNT: usize = 0;
static mut STRONG_PASSWORD_IDX: usize = 0;
static mut MATCH_NUMBER: usize = 0;
static mut CHECKED: bool = false;

const USAGE_MSG: &str =
    "Usage: ./uqentropy [--leet] [--double] [--digit-append 1..8] [--case] [listfilename ...]";

#[derive(Debug)]
enum ExitCodes {
    Usage = 2,
    InvalidFile = 20,
    NoStrong = 14,
}

// -------------------------------
// clap 命令行定义
// -------------------------------
#[derive(Parser, Debug)]
#[command(
    name = "uqentropy",
    version,
    about = "Password entropy checker",
    long_about = "UQEntropy — check password entropy and strength"
)]
struct Cli {
    /// Enable leet substitutions
    #[arg(long)]
    leet: bool,

    /// Enable case-sensitive comparison
    #[arg(long)]
    case: bool,

    /// Enable double password combination check
    #[arg(long)]
    double: bool,

    /// Enable digit append (1..8)
    #[arg(long = "digit-append")]
    digit_append: Option<usize>,

    /// One or more password list files
    #[arg()]
    list_filenames: Vec<String>,
}

// -------------------------------
// 逻辑函数
// -------------------------------
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

        for line in reader.lines() {
            if let Ok(l) = line {
                for token in l.split_whitespace() {
                    if !token.is_empty() {
                        passwords.push(token.to_string());
                        unsafe {
                            PASSWORD_COUNT += 1;
                        }
                    }
                }
            }
        }

        if unsafe { PASSWORD_COUNT == 0 } {
            eprintln!("uqentropy: \"{}\" does not contain any passwords", fname);
            found_error = true;
        }
    }

    if found_error {
        exit(ExitCodes::InvalidFile as i32);
    }
}

fn check_password_is_valid(password: &str) -> bool {
    !password.is_empty() && password.chars().all(|c| c.is_ascii_graphic() && !c.is_whitespace())
}

// -------------------------------
// 主函数（使用 clap 替代手动解析）
// -------------------------------
fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        if arg.is_empty() {
            eprintln!("{}", USAGE_MSG);
            std::process::exit(2);
        }
    }
    let cli = Cli::parse();

    // 参数验证
    if let Some(n) = cli.digit_append {
        if n < 1 || n > 8 {
            eprintln!("{}", USAGE_MSG);
            exit(ExitCodes::Usage as i32);
        }
        unsafe {
            NUM_DIGITS = n;
            DIGIT_APPEND = true;
        }
    }

    if cli.list_filenames.is_empty(){
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }
    // 更新全局变量
    unsafe {
        LEET = cli.leet;
        CASE_SENSITIVE = cli.case;
        DOUBLE_CHECK = cli.double;
    }

    if (unsafe { LEET || CASE_SENSITIVE || DIGIT_APPEND || DOUBLE_CHECK }) && cli.list_filenames.is_empty() {
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }

    let mut passwords: Vec<String> = Vec::new();
    read_file(&cli.list_filenames, &mut passwords);

    println!("Welcome to UQEntropy!");
    println!("Written by s4905773.");
    println!("Enter candidate passwords to check their strength.");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let mut password = line.unwrap();
        password = password.trim_end().to_string();

        if !check_password_is_valid(&password) {
            eprintln!("Password is invalid");
            continue;
        }

        let entropy = calculate_entropy(&password);
        println!("Password entropy calculated to be {:.1}", entropy);
        println!("Password strength rating: {}", map_to_strength(entropy));
    }
}
