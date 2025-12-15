use a1_2024_s2::*;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::process::Command;

// 测试calculate_entropy函数
#[test]
fn test_calculate_entropy() {
    // 测试空字符串
    assert_eq!(calculate_entropy(""), 0.0);

    // 测试仅小写字母
    assert_eq!(calculate_entropy("password"), 37.6);

    // 测试仅大写字母
    assert_eq!(calculate_entropy("PASSWORD"), 37.6);

    // 测试大小写混合
    assert_eq!(calculate_entropy("Password"), 45.6);

    // 测试包含数字
    assert_eq!(calculate_entropy("Password123"), 65.4);

    // 测试包含符号
    assert_eq!(calculate_entropy("Password!"), 57.5);

    // 测试包含数字和符号
    assert_eq!(calculate_entropy("Password123!"), 78.6);
}

// 测试map_to_strength函数
#[test]
fn test_map_to_strength() {
    // 测试very weak
    assert_eq!(map_to_strength(30.0), "very weak");
    assert_eq!(map_to_strength(34.9), "very weak");

    // 测试weak
    assert_eq!(map_to_strength(35.0), "weak");
    assert_eq!(map_to_strength(59.9), "weak");

    // 测试strong
    assert_eq!(map_to_strength(60.0), "strong");
    assert_eq!(map_to_strength(119.9), "strong");

    // 测试very strong
    assert_eq!(map_to_strength(120.0), "very strong");
    assert_eq!(map_to_strength(200.0), "very strong");
}

// 测试check_password_is_valid函数
#[test]
fn test_check_password_is_valid() {
    // 测试空字符串
    assert!(!check_password_is_valid(""));

    // 测试有效密码
    assert!(check_password_is_valid("password"));
    assert!(check_password_is_valid("Password123!"));

    // 测试包含空格的密码
    assert!(check_password_is_valid("my password"));

    // 测试包含换行符的密码（根据实际逻辑，这是允许的）
    assert!(check_password_is_valid("password\n"));

    // 测试包含制表符的密码（根据实际逻辑，这是允许的）
    assert!(check_password_is_valid("password\t"));

    // 测试包含空字符控制字符的密码（这是不允许的）
    assert!(!check_password_is_valid("password\x00"));

    // 测试包含非ASCII字符的密码
    assert!(!check_password_is_valid("密码"));
    assert!(!check_password_is_valid("password@测试"));
}

// 测试floor_to_one_decimal函数
#[test]
fn test_floor_to_one_decimal() {
    assert_eq!(floor_to_one_decimal(10.0), 10.0);
    assert_eq!(floor_to_one_decimal(10.1), 10.1);
    assert_eq!(floor_to_one_decimal(10.12), 10.1);
    assert_eq!(floor_to_one_decimal(10.19), 10.1);
    assert_eq!(floor_to_one_decimal(10.99), 10.9);
}

// 测试get_letter_count函数
#[test]
fn test_get_letter_count() {
    assert_eq!(get_letter_count(""), 0);
    assert_eq!(get_letter_count("12345"), 0);
    assert_eq!(get_letter_count("password"), 8);
    assert_eq!(get_letter_count("Password123!"), 8);
    assert_eq!(get_letter_count("a1b2c3d4"), 4);
}

// CLI测试 - 测试使用无效参数时的错误输出
#[test]
fn test_cli_invalid_args() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("");
    cmd.assert()
        .stderr(predicate::str::contains("Usage: ./uqentropy [--leet] [--double] [--digit-append 1..8] [--case] [listfilename ...]")
        .and(predicate::str::is_empty().not()))
        .code(2);
}

// CLI测试 - 测试使用选项但不提供文件时的错误输出
#[test]
fn test_cli_options_without_files() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("--leet");
    cmd.assert()
        .stderr(predicate::str::contains("Usage: ./uqentropy [--leet] [--double] [--digit-append 1..8] [--case] [listfilename ...]")
        .and(predicate::str::is_empty().not()))
        .code(2);
}

// CLI测试 - 测试使用有效文件
#[test]
fn test_cli_with_valid_file() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("testfiles/top25.passwords");

    // 向命令发送输入 - 使用强密码
    cmd.write_stdin("StrongPassword123!\n");

    cmd.assert()
        .stdout(predicate::str::contains(
            "Password entropy calculated to be",
        ))
        .stdout(predicate::str::contains("Password strength rating: strong"))
        .code(0);
}

// CLI测试 - 测试使用--case选项
#[test]
fn test_cli_with_case_option() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("--case");
    cmd.arg("testfiles/top25.passwords");

    // 向命令发送输入 - 使用强密码
    cmd.write_stdin("StrongPassword123!\n");

    cmd.assert()
        .stdout(predicate::str::contains(
            "Password entropy calculated to be",
        ))
        .code(0);
}

// CLI测试 - 测试使用--digit-append选项
#[test]
fn test_cli_with_digit_append_option() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("--digit-append");
    cmd.arg("3");
    cmd.arg("testfiles/top25.passwords");

    // 向命令发送输入 - 使用强密码
    cmd.write_stdin("StrongPassword123!\n");

    cmd.assert()
        .stdout(predicate::str::contains(
            "Password entropy calculated to be",
        ))
        .code(0);
}

// CLI测试 - 测试使用--leet选项
#[test]
fn test_cli_with_leet_option() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("--leet");
    cmd.arg("testfiles/top25.passwords");

    // 向命令发送输入 - 使用强密码
    cmd.write_stdin("StrongPassword123!\n");

    cmd.assert()
        .stdout(predicate::str::contains(
            "Password entropy calculated to be",
        ))
        .code(0);
}

// CLI测试 - 测试使用--double选项
#[test]
fn test_cli_with_double_option() {
    let mut cmd = cargo_bin_cmd!("uqentropy");
    cmd.arg("--double");
    cmd.arg("testfiles/top25.passwords");

    // 向命令发送输入 - 使用强密码
    cmd.write_stdin("StrongPassword123!\n");

    cmd.assert()
        .stdout(predicate::str::contains(
            "Password entropy calculated to be",
        ))
        .code(0);
}
