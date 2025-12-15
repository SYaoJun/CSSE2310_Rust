use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::{self, Write};
use tempfile::NamedTempFile;

#[test]
fn test_welcome_message() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.assert()
        .stdout(predicate::str::contains("Welcome to uqbasejump!"))
        .stdout(predicate::str::contains("Input base set to: 10"))
        .stdout(predicate::str::contains("Output bases: 2, 10, 16"))
        .stdout(predicate::str::contains(
            "Please enter your numbers and expressions.",
        ))
        .stdout(predicate::str::contains("Thanks for using uqbasejump."))
        .success();
}

#[test]
fn test_single_number() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("42\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): "))
        .stdout(predicate::str::contains("Input (base 10): 42"))
        .stdout(predicate::str::contains("Base 2: 101010"))
        .stdout(predicate::str::contains("Base 10: 42"))
        .stdout(predicate::str::contains("Base 16: 2A"))
        .success();
}

#[test]
fn test_basic_arithmetic() {
    // Addition
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10+5\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 10+5"))
        .stdout(predicate::str::contains("Result (base 10): 15"))
        .stdout(predicate::str::contains("Base 2: 1111"))
        .stdout(predicate::str::contains("Base 10: 15"))
        .stdout(predicate::str::contains("Base 16: F"))
        .success();

    // Subtraction
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10-5\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 10-5"))
        .stdout(predicate::str::contains("Result (base 10): 5"))
        .stdout(predicate::str::contains("Base 2: 101"))
        .stdout(predicate::str::contains("Base 10: 5"))
        .stdout(predicate::str::contains("Base 16: 5"))
        .success();

    // Multiplication
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10*5\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 10*5"))
        .stdout(predicate::str::contains("Result (base 10): 50"))
        .stdout(predicate::str::contains("Base 2: 110010"))
        .stdout(predicate::str::contains("Base 10: 50"))
        .stdout(predicate::str::contains("Base 16: 32"))
        .success();

    // Division
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10/5\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 10/5"))
        .stdout(predicate::str::contains("Result (base 10): 2"))
        .stdout(predicate::str::contains("Base 2: 10"))
        .stdout(predicate::str::contains("Base 10: 2"))
        .stdout(predicate::str::contains("Base 16: 2"))
        .success();
}

#[test]
fn test_operator_precedence() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("2+3*4\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 2+3*4"))
        .stdout(predicate::str::contains("Result (base 10): 14"))
        .stdout(predicate::str::contains("Base 2: 1110"))
        .stdout(predicate::str::contains("Base 10: 14"))
        .stdout(predicate::str::contains("Base 16: E"))
        .success();

    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("2*3+4\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 2*3+4"))
        .stdout(predicate::str::contains("Result (base 10): 10"))
        .stdout(predicate::str::contains("Base 2: 1010"))
        .stdout(predicate::str::contains("Base 10: 10"))
        .stdout(predicate::str::contains("Base 16: A"))
        .success();

    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10-5/5\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 10-5/5"))
        .stdout(predicate::str::contains("Result (base 10): 9"))
        .stdout(predicate::str::contains("Base 2: 1001"))
        .stdout(predicate::str::contains("Base 10: 9"))
        .stdout(predicate::str::contains("Base 16: 9"))
        .success();
}

#[test]
fn test_large_number() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("123456789\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input (base 10): 123456789"))
        .stdout(predicate::str::contains("Base 2:"))
        .stdout(predicate::str::contains("Base 10: 123456789"))
        .stdout(predicate::str::contains("Base 16:"))
        .success();
}

#[test]
fn test_binary_input() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("2").write_stdin("1010\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input base set to: 2"))
        .stdout(predicate::str::contains("Input (base 2): 1010"))
        .stdout(predicate::str::contains("Base 2: 1010"))
        .stdout(predicate::str::contains("Base 10: 10"))
        .stdout(predicate::str::contains("Base 16: A"))
        .success();
}

#[test]
fn test_hex_input() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("16").write_stdin("FF\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input base set to: 16"))
        .stdout(predicate::str::contains("Input (base 16): FF"))
        .stdout(predicate::str::contains("Base 2: 11111111"))
        .stdout(predicate::str::contains("Base 10: 255"))
        .stdout(predicate::str::contains("Base 16: FF"))
        .success();
}

#[test]
fn test_custom_output_bases() {
    // Test with base 8
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--obases").arg("8").write_stdin("10\n");
    cmd.assert()
        .stdout(predicate::str::contains("Output bases: 8"))
        .stdout(predicate::str::contains("Base 8: 12"))
        .success();

    // Test with multiple bases
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--obases").arg("3,7,10").write_stdin("20\n");
    cmd.assert()
        .stdout(predicate::str::contains("Output bases: 3, 7, 10"))
        .stdout(predicate::str::contains("Base 3: 202"))
        .stdout(predicate::str::contains("Base 7: 26"))
        .stdout(predicate::str::contains("Base 10: 20"))
        .success();
}

#[test]
fn test_input_file() {
    // Create a temporary file with input
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "7231").unwrap();
    writeln!(temp_file, "3*7").unwrap();
    temp_file.flush().unwrap();

    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inputfile")
        .arg(temp_file.path().to_str().unwrap());
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 7231"))
        .stdout(predicate::str::contains("Result (base 10): 7231"))
        .stdout(predicate::str::contains("Expression (base 10): 3*7"))
        .stdout(predicate::str::contains("Result (base 10): 21"))
        .success();
}

#[test]
fn test_invalid_input() {
    // Incomplete expression
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("5+\n");
    cmd.assert()
        .stderr(predicate::str::contains(
            "Can't evaluate the expression \"5+\"",
        ))
        .success();

    // Invalid expression format - operator at start
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("+5\n");
    cmd.assert()
        .stderr(predicate::str::contains(
            "Can't evaluate the expression \"+5\"",
        ))
        .success();

    // Invalid expression format - consecutive operators
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("5++3\n");
    cmd.assert()
        .stderr(predicate::str::contains(
            "Can't evaluate the expression \"5++3\"",
        ))
        .success();
}

#[test]
fn test_division_by_zero() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10/0\n");
    cmd.assert()
        .stderr(predicate::str::contains(
            "Can't evaluate the expression \"10/0\"",
        ))
        .success();
}

#[test]
fn test_overflow() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    // u128 maximum is 340282366920938463463374607431768211455, so 340282366920938463463374607431768211455+1 should overflow
    cmd.write_stdin("340282366920938463463374607431768211455+1\n");
    cmd.assert()
        .stderr(predicate::str::contains(
            "Can't evaluate the expression \"340282366920938463463374607431768211455+1\"",
        ))
        .success();
}

#[test]
fn test_subtraction_underflow() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("5-10\n");
    cmd.assert()
        .stderr(predicate::str::contains(
            "Can't evaluate the expression \"5-10\"",
        ))
        .success();
}

#[test]
fn test_combined_options() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase")
        .arg("8")
        .arg("--obases")
        .arg("2,10")
        .write_stdin("50\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input base set to: 8"))
        .stdout(predicate::str::contains("Output bases: 2, 10"))
        .stdout(predicate::str::contains("Input (base 8): 50"))
        .stdout(predicate::str::contains("Base 2: 101000"))
        .stdout(predicate::str::contains("Base 10: 40"))
        .success();
}

#[test]
fn test_invalid_base_options() {
    // Invalid input base
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("1").write_stdin("10\n");
    cmd.assert()
        .code(7) // Usage error
        .stderr(predicate::str::contains("Usage:"))
        .failure();

    // Invalid output base
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--obases").arg("37").write_stdin("10\n");
    cmd.assert()
        .code(7) // Usage error
        .stderr(predicate::str::contains("Usage:"))
        .failure();

    // Invalid base format
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("abc").write_stdin("10\n");
    cmd.assert()
        .code(7) // Usage error
        .stderr(predicate::str::contains("Usage:"))
        .failure();
}

#[test]
fn test_file_not_found() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inputfile").arg("nonexistent_file.txt");
    cmd.assert()
        .code(16) // File error
        .stderr(predicate::str::contains(
            "unable to read from file \"nonexistent_file.txt\"",
        ))
        .failure();
}

#[test]
fn test_backspace() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    // Input "123" then backspace once to get "12"
    cmd.write_stdin("123\x7f4\n");
    // We can't easily test the intermediate state, but the final result should be 124
    cmd.assert()
        .stdout(predicate::str::contains("Input (base 10): 124"))
        .stdout(predicate::str::contains("Base 10: 124"))
        .success();
}

#[test]
fn test_escape() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    // Input "123", escape, then "45"
    cmd.write_stdin("123\x1b45\n");
    // The escape should clear the input, so we should only see "45"
    cmd.assert()
        .stdout(predicate::str::contains("Input (base 10): 45"))
        .stdout(predicate::str::contains("Base 10: 45"))
        .success();
}

#[test]
fn test_empty_input_file() {
    // Create an empty temporary file
    let temp_file = NamedTempFile::new().unwrap();

    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inputfile")
        .arg(temp_file.path().to_str().unwrap());
    // Empty file should not produce any errors, just process nothing
    cmd.assert().success();
}

#[test]
fn test_uppercase_hex_input() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("16").write_stdin("ABC\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input (base 16): ABC"))
        .stdout(predicate::str::contains("Base 10: 2748"))
        .success();
}

#[test]
fn test_lowercase_hex_input() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("16").write_stdin("abc\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input (base 16): abc"))
        .stdout(predicate::str::contains("Base 10: 2748"))
        .success();
}

#[test]
fn test_octal_input() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("8").write_stdin("177\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input base set to: 8"))
        .stdout(predicate::str::contains("Input (base 8): 177"))
        .stdout(predicate::str::contains("Base 10: 127"))
        .success();
}

#[test]
fn test_max_base() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.arg("--inbase").arg("36").write_stdin("zzz\n");
    cmd.assert()
        .stdout(predicate::str::contains("Input base set to: 36"))
        .stdout(predicate::str::contains("Input (base 36): zzz"))
        .success();
}

#[test]
fn test_multiple_expressions() {
    let mut cmd = cargo_bin_cmd!("uqbasejump");
    cmd.write_stdin("10+5\n20-3\n4*6\n");
    cmd.assert()
        .stdout(predicate::str::contains("Expression (base 10): 10+5"))
        .stdout(predicate::str::contains("Result (base 10): 15"))
        .stdout(predicate::str::contains("Expression (base 10): 20-3"))
        .stdout(predicate::str::contains("Result (base 10): 17"))
        .stdout(predicate::str::contains("Expression (base 10): 4*6"))
        .stdout(predicate::str::contains("Result (base 10): 24"))
        .success();
}

#[test]
fn test_consecutive_operators() {
    // Test various consecutive operator combinations
    let test_cases = [
        "5++3", "5--3", "5**3", "5//3", "5+-3", "5-+3", "5*+3", "5/-3",
    ];

    for &test_case in &test_cases {
        let mut cmd = cargo_bin_cmd!("uqbasejump");
        cmd.write_stdin(format!("{}\n", test_case));
        cmd.assert()
            .stderr(predicate::str::contains(format!(
                "Can't evaluate the expression \"{}\"",
                test_case
            )))
            .success();
    }
}
