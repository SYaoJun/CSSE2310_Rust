use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::num::ParseIntError;

use anyhow::{bail, Result};
use thiserror::Error;

const ADD: char = '+';
const SUBTRACT: char = '-';
const MULTIPLY: char = '*';
const DIVIDE: char = '/';
const CHANGE_INPUT_BASE: char = 'i';
const CHANGE_OUTPUT_BASE: char = 'o';
const SHOW_HISTORY: char = 'h';
const ZERO: char = '0';
const COMMA: char = ',';

const DECIMAL_BASE: u32 = 10;
const DEFAULT_OUTPUT_BASES: [u32; 3] = [2, 10, 16];
const MIN_BASE: u32 = 2;
const MAX_BASE: u32 = 36;

const PRINT_EXPRESSION_STR_BASE: &str = "Expression (base %s): %s\n";
const PRINT_RESULT_STRBASE: &str = "Result (base %s): %s\n";
const EXPRESSION_ERROR: &str = "Can't evaluate the expression \"%s\"\n";

const WELCOME_MESSAGE: &str = "Welcome to uqbasejump!\n@yaojun wrote this program.\n";
const WELCOME_INPUT_BASE: &str = "Input base set to: ";
const WELCOME_OUTPUT_BASE: &str = "Output bases: ";
const WELCOME_LAST_LINE: &str = "Please enter your numbers and expressions.\n";

const INPUT_BASE_ARG: &str = "inbase";
const OUTPUT_BASE_ARG: &str = "obases";
const INPUT_FILE_ARG: &str = "inputfile";

const OK_EXIT_MESSAGE: &str = "Thanks for using uqbasejump.\n";
const USAGE_ERROR_MESSAGE: &str =
    "Usage: ./uqbasejump [--obases 2..36] [--inbase 2..36] [--inputfile string]\n";
const FILE_ERROR_MESSAGE: &str = "uqbasejump: unable to read from file \"%s\"\n";

#[derive(Debug, Error)]
enum ExitError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File(String),
}

#[derive(Clone)]
struct Arguments {
    input_base: u32,
    output_bases: Vec<u32>,
    input_file_name: Option<String>,
}

#[derive(Default)]
struct InputExpr {
    input: String,
    expr: String,
    history: Vec<(String, String, u32)>,
}

fn main() {
    if let Err(err) = run() {
        match err.downcast_ref::<ExitError>() {
            Some(ExitError::Usage) => {
                eprint!("{USAGE_ERROR_MESSAGE}");
                std::process::exit(7);
            }
            Some(ExitError::File(path)) => {
                eprint!("{}", FILE_ERROR_MESSAGE.replace("%s", path));
                std::process::exit(16);
            }
            None => {
                eprintln!("{err:#}");
                std::process::exit(1);
            }
        }
    }
}

fn run() -> Result<()> {
    let args = parse_command_line().map_err(|e| match e {
        ExitError::Usage => anyhow::Error::from(ExitError::Usage),
        ExitError::File(path) => anyhow::Error::from(ExitError::File(path)),
    })?;

    if let Some(path) = args.input_file_name.clone() {
        let file = File::open(&path).map_err(|_| ExitError::File(path.clone()))?;
        let reader = BufReader::new(file);
        print_welcome_message(&args, false);
        process_file(reader, &args)?;
        print_ok_and_exit();
    } else {
        print_welcome_message(&args, true);
        process_stdin(&args)?;
        print_ok_and_exit();
    }

    Ok(())
}

fn parse_command_line() -> Result<Arguments, ExitError> {
    let mut argv: Vec<String> = env::args().collect();
    if argv.is_empty() {
        return Err(ExitError::Usage);
    }
    argv.remove(0); // program name

    let mut args = Arguments {
        input_base: DECIMAL_BASE,
        output_bases: DEFAULT_OUTPUT_BASES.to_vec(),
        input_file_name: None,
    };
    let mut input_base_set = false;
    let mut output_base_set = false;
    let mut input_file_set = false;

    let mut i = 0;
    while i < argv.len() {
        let token = &argv[i];
        if !token.starts_with("--") || token.len() <= 2 {
            return Err(ExitError::Usage);
        }
        let key = &token[2..];
        match key {
            INPUT_BASE_ARG => {
                if input_base_set {
                    return Err(ExitError::Usage);
                }
                i += 1;
                let val = argv.get(i).ok_or(ExitError::Usage)?;
                args.input_base = check_base(val).ok_or(ExitError::Usage)?;
                input_base_set = true;
            }
            OUTPUT_BASE_ARG => {
                if output_base_set {
                    return Err(ExitError::Usage);
                }
                i += 1;
                let val = argv.get(i).ok_or(ExitError::Usage)?;
                args.output_bases = parse_output_bases(val).map_err(|_| ExitError::Usage)?;
                output_base_set = true;
            }
            INPUT_FILE_ARG => {
                if input_file_set {
                    return Err(ExitError::Usage);
                }
                i += 1;
                let val = argv.get(i).ok_or(ExitError::Usage)?;
                if val.is_empty() {
                    return Err(ExitError::Usage);
                }
                args.input_file_name = Some(val.clone());
                input_file_set = true;
            }
            _ => return Err(ExitError::Usage),
        }
        i += 1;
    }

    Ok(args)
}

fn parse_output_bases(src: &str) -> Result<Vec<u32>, ParseIntError> {
    let mut bases = Vec::new();
    for part in src.split(COMMA) {
        if part.is_empty() {
            return Err("".parse::<u32>().unwrap_err());
        }
        // If check_base() failed (e.g. value out of 2..=36 or non-digit), fabricate
        // a ParseIntError using an intentionally invalid parse so we can bubble up
        // an Err instead of panicking (previously unwrap_err on "0" would panic
        // because it parses successfully).
        let val = check_base(part).ok_or_else(|| "".parse::<u32>().unwrap_err())?;
        if bases.contains(&val) {
            return Err("".parse::<u32>().unwrap_err());
        }
        bases.push(val);
    }
    Ok(bases)
}

fn check_base(base_str: &str) -> Option<u32> {
    if base_str.is_empty() {
        return None;
    }
    let value: u32 = base_str.parse().ok()?;
    if value < MIN_BASE || value > MAX_BASE {
        return None;
    }
    if !base_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(value)
}

fn print_welcome_message(args: &Arguments, show_prompt: bool) {
    print!("{WELCOME_MESSAGE}");
    println!("{WELCOME_INPUT_BASE}{}", args.input_base);
    print!("{WELCOME_OUTPUT_BASE}");
    for (idx, base) in args.output_bases.iter().enumerate() {
        if idx + 1 == args.output_bases.len() {
            println!("{base}");
        } else {
            print!("{base}, ");
        }
    }
    if show_prompt {
        print!("{WELCOME_LAST_LINE}");
    }
    let _ = io::stdout().flush();
}

fn print_ok_and_exit() {
    print!("{OK_EXIT_MESSAGE}");
    let _ = io::stdout().flush();
}

fn process_file<R: BufRead>(mut reader: R, args: &Arguments) -> Result<()> {
    let mut line = String::new();
    while reader.read_line(&mut line)? != 0 {
        let trimmed = line.trim_end_matches('\n');
        let expr_base10 = convert_expression(trimmed, args.input_base, DECIMAL_BASE)?;
        match evaluate_expression(&expr_base10) {
            Ok(result) => {
                let expr_input = convert_expression(trimmed, args.input_base, args.input_base)?;
                let result_conv = convert_int_to_str_any_base(result, args.input_base);
                print_expression(args.input_base, &expr_input);
                print_result(args.input_base, &result_conv);
                print_in_bases(result, args);
            }
            Err(_) => eprint!("{}", EXPRESSION_ERROR.replace("%s", trimmed)),
        }
        line.clear();
    }
    Ok(())
}

fn process_stdin(args: &Arguments) -> Result<()> {
    let mut input_expr = InputExpr::default();
    let stdin = io::stdin();
    loop {
        let mut buffer = String::new();
        let read = stdin.read_line(&mut buffer)?;
        if read == 0 {
            break;
        }
        let line = buffer.trim_end_matches('\n');
        if line == "\u{4}" || line == "\u{1b}" {
            break;
        }
        if line.starts_with(':') {
            handle_command_line(&mut input_expr, args, line)?;
            continue;
        }
        if line.is_empty() {
            continue;
        }
        input_expr.input = line.to_string();
        process_expression(&mut input_expr, args)?;
    }
    Ok(())
}

fn handle_command_line(input_expr: &mut InputExpr, args: &Arguments, line: &str) -> Result<()> {
    let mut chars = line.chars();
    chars.next(); // drop ':'
    let cmd = chars.next().unwrap_or_default();
    let rest = chars.as_str().trim();
    match cmd {
        CHANGE_INPUT_BASE => {
            let base = check_base(rest).ok_or(ExitError::Usage)?;
            input_expr.input.clear();
            input_expr.expr.clear();
            let mut new_args = args.clone();
            new_args.input_base = base;
            println!("{WELCOME_INPUT_BASE}{base}");
        }
        CHANGE_OUTPUT_BASE => {
            let bases = parse_output_bases(rest).map_err(|_| ExitError::Usage)?;
            let mut new_args = args.clone();
            new_args.output_bases = bases;
            println!("{WELCOME_OUTPUT_BASE}{:?}", new_args.output_bases);
        }
        SHOW_HISTORY => {
            for (expr, result, base) in &input_expr.history {
                println!(
                    "{}",
                    PRINT_EXPRESSION_STR_BASE.replace("%s", &base.to_string()).replace("%s", expr)
                );
                println!(
                    "{}",
                    PRINT_RESULT_STRBASE.replace("%s", &base.to_string()).replace("%s", result)
                );
            }
        }
        _ => {}
    }
    Ok(())
}

fn process_expression(input_expr: &mut InputExpr, args: &Arguments) -> Result<()> {
    if input_expr.input.is_empty() {
        input_expr.input.push(ZERO);
    }
    let base_ten_input = convert_any_base_to_base_ten(&input_expr.input, args.input_base)?;
    input_expr.expr.push_str(&base_ten_input);
    let result = match evaluate_expression(&input_expr.expr) {
        Ok(res) => res,
        Err(_) => {
            eprint!("{}", EXPRESSION_ERROR.replace("%s", &input_expr.expr));
            input_expr.expr.clear();
            input_expr.input.clear();
            return Ok(());
        }
    };

    let expr_converted = convert_expression(&input_expr.expr, DECIMAL_BASE, args.input_base)?;
    println!("Expression (base {}): {}", args.input_base, expr_converted);
    let result_converted = convert_int_to_str_any_base(result, args.input_base);
    println!("Result (base {}): {}", args.input_base, result_converted);
    print_in_bases(result, args);
    input_expr.history.push((expr_converted, result_converted, args.input_base));
    input_expr.expr.clear();
    input_expr.input.clear();
    Ok(())
}

fn print_expression(base: u32, expr: &str) {
    println!("Expression (base {}): {}", base, expr);
}

fn print_result(base: u32, result: &str) {
    println!("Result (base {}): {}", base, result);
}

fn print_in_bases(value: u128, args: &Arguments) {
    for base in &args.output_bases {
        let result = convert_int_to_str_any_base(value, *base);
        println!("Base {}: {}", base, result);
    }
}

fn convert_expression(expr: &str, from_base: u32, to_base: u32) -> Result<String> {
    let mut out = String::new();
    let mut current = String::new();
    for ch in expr.chars() {
        if is_operator(ch) {
            if !current.is_empty() {
                let converted = convert_any_base_to_base(current.clone(), from_base, to_base)?;
                out.push_str(&converted);
                current.clear();
            }
            out.push(ch);
        } else if !ch.is_whitespace() {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        let converted = convert_any_base_to_base(current, from_base, to_base)?;
        out.push_str(&converted);
    }
    Ok(out)
}

fn is_operator(ch: char) -> bool {
    matches!(ch, ADD | SUBTRACT | MULTIPLY | DIVIDE)
}

fn convert_any_base_to_base_ten(num: &str, base: u32) -> Result<String> {
    let value = convert_str_to_int_any_base(num, base)?;
    Ok(value.to_string())
}

fn convert_any_base_to_base(num: String, from: u32, to: u32) -> Result<String> {
    let val = convert_str_to_int_any_base(&num, from)?;
    Ok(convert_int_to_str_any_base(val, to))
}

fn convert_str_to_int_any_base(num: &str, base: u32) -> Result<u128> {
    let mut value: u128 = 0;
    for c in num.chars() {
        let digit = char_to_digit(c).ok_or_else(|| anyhow::anyhow!("invalid digit"))?;
        if digit >= base as u8 {
            bail!("digit out of range");
        }
        value = value
            .checked_mul(base as u128)
            .and_then(|v| v.checked_add(digit as u128))
            .ok_or_else(|| anyhow::anyhow!("overflow"))?;
    }
    Ok(value)
}

fn convert_int_to_str_any_base(mut value: u128, base: u32) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut chars = VecDeque::new();
    while value > 0 {
        let rem = (value % base as u128) as u8;
        chars.push_front(digit_to_char(rem));
        value /= base as u128;
    }
    chars.iter().collect()
}

fn char_to_digit(c: char) -> Option<u8> {
    if c.is_ascii_digit() {
        Some(c as u8 - b'0')
    } else if c.is_ascii_lowercase() {
        Some(10 + c as u8 - b'a')
    } else if c.is_ascii_uppercase() {
        Some(10 + c as u8 - b'A')
    } else {
        None
    }
}

fn digit_to_char(d: u8) -> char {
    match d {
        0..=9 => (b'0' + d) as char,
        _ => (b'a' + (d - 10)) as char,
    }
}

fn evaluate_expression(expr: &str) -> Result<u128> {
    let tokens = tokenize(expr)?;
    let rpn = to_rpn(tokens)?;
    eval_rpn(&rpn)
}

#[derive(Debug, Clone)]
enum Token {
    Number(u128),
    Op(char),
}

fn tokenize(expr: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for c in expr.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else if is_operator(c) {
            if num.is_empty() {
                bail!("invalid expression");
            }
            let val: u128 = num.parse()?;
            tokens.push(Token::Number(val));
            num.clear();
            tokens.push(Token::Op(c));
        } else if c.is_whitespace() {
            continue;
        } else {
            bail!("invalid character");
        }
    }
    if !num.is_empty() {
        let val: u128 = num.parse()?;
        tokens.push(Token::Number(val));
    }
    Ok(tokens)
}

fn precedence(op: char) -> u8 {
    match op {
        ADD | SUBTRACT => 1,
        MULTIPLY | DIVIDE => 2,
        _ => 0,
    }
}

fn to_rpn(tokens: Vec<Token>) -> Result<Vec<Token>> {
    let mut output = Vec::new();
    let mut stack: Vec<char> = Vec::new();
    for tok in tokens {
        match tok {
            Token::Number(_) => output.push(tok),
            Token::Op(op) => {
                while let Some(&top) = stack.last() {
                    if precedence(top) >= precedence(op) {
                        output.push(Token::Op(top));
                        stack.pop();
                    } else {
                        break;
                    }
                }
                stack.push(op);
            }
        }
    }
    while let Some(op) = stack.pop() {
        output.push(Token::Op(op));
    }
    Ok(output)
}

fn eval_rpn(tokens: &[Token]) -> Result<u128> {
    let mut stack: Vec<u128> = Vec::new();
    for tok in tokens {
        match tok {
            Token::Number(n) => stack.push(*n),
            Token::Op(op) => {
                if stack.len() < 2 {
                    bail!("invalid expression");
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let res = match *op {
                    ADD => a.checked_add(b),
                    SUBTRACT => a.checked_sub(b),
                    MULTIPLY => a.checked_mul(b),
                    DIVIDE => {
                        if b == 0 {
                            None
                        } else {
                            a.checked_div(b)
                        }
                    }
                    _ => None,
                }
                .ok_or_else(|| anyhow::anyhow!("overflow or invalid op"))?;
                stack.push(res);
            }
        }
    }
    if stack.len() == 1 {
        Ok(stack[0])
    } else {
        bail!("invalid expression")
    }
}
