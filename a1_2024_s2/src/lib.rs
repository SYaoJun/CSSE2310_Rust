pub mod utils;

// Export core functions from uqentropy for testing
pub struct Config {
    pub leet: bool,
    pub case_sensitive: bool,
    pub digit_append: bool,
    pub double_check: bool,
    pub num_digits: usize,
}

impl Config {
    pub fn new() -> Self {
        Config {
            leet: false,
            case_sensitive: false,
            digit_append: false,
            double_check: false,
            num_digits: 0,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// 添加自定义log2函数，与原始代码保持一致
fn log2(x: f64) -> f64 {
    x.log2()
}

pub fn calculate_entropy(password: &str) -> f64 {
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

    // 处理s为0的情况，避免log2(0)产生NaN
    if s == 0 {
        return 0.0;
    }

    let result = password.len() as f64 * log2(s as f64);
    floor_to_one_decimal(result)
}

pub fn map_to_strength(entropy: f64) -> &'static str {
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

pub fn check_password_is_valid(password: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    // 只允许可打印的ASCII字符，不包括控制字符
    password
        .chars()
        .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
}

pub fn floor_to_one_decimal(x: f64) -> f64 {
    (x * 10.0).floor() / 10.0
}

pub fn get_letter_count(s: &str) -> i32 {
    let mut count = 0;
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            count += 1;
        }
    }
    count
}
