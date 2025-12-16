use anyhow::Result;
use std::f64::consts::{E, PI};
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExitError {
    #[error("usage")]
    Usage,
    #[error("file")]
    File,
    #[error("variable")]
    Variable,

    #[error("duplicate variable")]
    Duplicate,
}
#[derive(Debug)]
pub enum ExitCodes {
    Usage = 18,
    InvalidFile = 5,
    Variable = 14,
    Duplicate = 19,
}
pub const DUPLICATE_MSG: &str = "uqexpr: duplicate variables were detected";
pub const USAGE_MSG: &str = "Usage: ./uqexpr [--init string] [--significantfigures 2..8] [--forloop string] [inputfilename]";
pub const VARIABLE_MSG: &str = "uqexpr: invalid variable(s) were specified";
// 是不是可以自己定义错误处理枚举类型
pub struct Config {
    pub init_string_vec: Vec<String>,
    pub init_order: Vec<String>,
    pub significant_figures: u8,
    pub for_loop_vec: Vec<String>,
    pub input_filename: String,
    pub figure_flag: bool,
    pub filename_flag: bool,
    pub init_map: std::collections::HashMap<String, f64>,
    pub for_loop_struct_vec: Vec<ForLoop>,
}
pub struct ForLoop {
    pub name: String,
    pub current: f64,
    pub start: f64,
    pub end: f64,
    pub increment: f64,
}

pub fn handle_command_line_arguments() -> Result<Config, ExitError> {
    let args: Vec<String> = std::env::args().collect();

    let mut config = Config {
        init_string_vec: Vec::new(),
        init_order: Vec::new(),
        significant_figures: 4,
        for_loop_vec: Vec::new(),
        input_filename: String::from(""),
        figure_flag: false,
        filename_flag: false,
        init_map: std::collections::HashMap::new(),
        for_loop_struct_vec: Vec::new(),
    };
    let mut i = 1;

    while i < args.len() {
        if args[i].is_empty() {
            return Err(ExitError::Usage);
        }
        match args[i].as_str() {
            "--init" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                config.init_string_vec.push(args[i + 1].clone());
                i += 2;
            }
            "--significantfigures" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                if config.figure_flag {
                    return Err(ExitError::Usage);
                }
                if has_leading_zero(args[i + 1].as_str()) {
                    return Err(ExitError::Usage);
                }
                config.figure_flag = true;
                let parse_int = args[i + 1].parse::<u8>();
                match parse_int {
                    Err(_) => {
                        return Err(ExitError::Usage);
                    }
                    Ok(x) => {
                        if x >= 2 && x <= 8 {
                            config.significant_figures = x;
                        } else {
                            return Err(ExitError::Usage);
                        }
                    }
                }
                i += 2;
            }
            "--forloop" => {
                if i + 1 >= args.len() {
                    return Err(ExitError::Usage);
                }
                config.for_loop_vec.push(args[i + 1].clone());
                i += 2;
            }
            _ => {
                if i != args.len() - 1 {
                    return Err(ExitError::Usage);
                }
                if args[i].starts_with("--") {
                    return Err(ExitError::Usage);
                }
                if config.filename_flag {
                    return Err(ExitError::Usage);
                }
                config.input_filename = args[i].clone();
                config.filename_flag = true;
                i += 1;
            }
        }
    }
    Ok(config)
}

pub fn has_leading_zero(s: &str) -> bool {
    if s.len() <= 1 {
        return false;
    }
    // Rust中字符串访问每个元素，需要转换为 Vec之后访问
    let chars: Vec<char> = s.chars().collect();

    if chars[0] == '0' {
        return true;
    }
    false
}

pub fn check_input_filename(filename: &String) -> Result<Vec<String>, ExitError> {
    let file = File::open(filename);
    if file.is_err() {
        return Err(ExitError::File);
    }
    let reader = BufReader::new(file.unwrap());
    let mut file_string: Vec<String> = vec![];
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                file_string.push(line);
            }
            Err(_) => {
                return Err(ExitError::File);
            }
        }
    }
    Ok(file_string)
}

// Helper function to validate variable names
fn is_valid_var_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 20 {
        return false;
    }

    // First character must be a letter
    if !name
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_alphabetic())
    {
        return false;
    }

    // Remaining characters must be alphanumeric or underscore
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// Helper function to validate numeric strings
fn is_valid_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut has_decimal = false;
    let mut has_digit = false;
    let mut chars = s.chars().peekable();

    // Handle optional sign
    if let Some('-') = chars.peek() {
        chars.next();
    }

    // Check each character
    while let Some(&c) = chars.peek() {
        if c == '.' {
            if has_decimal {
                return false; // Multiple decimal points
            }
            has_decimal = true;
        } else if !c.is_ascii_digit() {
            return false; // Invalid character
        } else {
            has_digit = true;
        }
        chars.next();
    }

    has_digit // Must have at least one digit
}

pub fn check_variable(config: &mut Config) -> Result<(), ExitError> {
    // Check for duplicate variable definitions in --init
    let mut seen_vars = std::collections::HashSet::new();

    for init_string in &config.init_string_vec {
        let parts: Vec<&str> = init_string.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(ExitError::Variable);
        }

        let var_name = parts[0].trim();
        let var_value = parts[1].trim();

        // Validate variable name
        if !is_valid_var_name(var_name) {
            return Err(ExitError::Variable);
        }

        // Check for duplicate variable names
        if !seen_vars.insert(var_name) {
            return Err(ExitError::Duplicate);
        }

        // Validate variable value
        if var_value.is_empty() || !is_valid_numeric(var_value) {
            return Err(ExitError::Variable);
        }

        // Parse and store the value
        let value = var_value.parse::<f64>().map_err(|_| ExitError::Variable)?;
        config.init_map.insert(var_name.to_string(), value);
        config.init_order.push(var_name.to_string());
    }

    // Process for loop variables
    for for_loop_str in &config.for_loop_vec {
        let parts: Vec<&str> = for_loop_str.split(',').collect();
        if parts.len() != 4 {
            return Err(ExitError::Variable);
        }

        // For-loop format is: name,start,increment,end
        let name = parts[0].trim();
        let start_str = parts[1].trim();
        let incr_str = parts[2].trim();
        let end_str = parts[3].trim();

        // Validate loop variable name
        if !is_valid_var_name(name) {
            return Err(ExitError::Variable);
        }

        // Check for duplicate loop variable names
        if !seen_vars.insert(name) {
            return Err(ExitError::Duplicate);
        }

        // Validate numeric values
        if !is_valid_numeric(start_str) || !is_valid_numeric(end_str) || !is_valid_numeric(incr_str)
        {
            return Err(ExitError::Variable);
        }

        // Parse numeric values
        let start = start_str.parse::<f64>().map_err(|_| ExitError::Variable)?;
        let increment = incr_str.parse::<f64>().map_err(|_| ExitError::Variable)?;
        let end = end_str.parse::<f64>().map_err(|_| ExitError::Variable)?;

        // Check for zero increment
        if increment == 0.0 {
            return Err(ExitError::Variable);
        }

        // Validate increment direction (must make progress toward end)
        if start < end {
            if increment < 0.0 {
                return Err(ExitError::Variable);
            }
        } else if start > end {
            if increment > 0.0 {
                return Err(ExitError::Variable);
            }
        }

        // Add to for_loop_struct_vec
        config.for_loop_struct_vec.push(ForLoop {
            name: name.to_string(),
            current: start,
            start,
            end,
            increment,
        });

        // Make loop variables available for expression evaluation.
        config.init_map.insert(name.to_string(), start);
    }

    Ok(())
}

// Token types for the expression parser
#[derive(Debug)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LeftParen,
    RightParen,
    Constant(String),
    Function(String),
    Variable(String),
    Equals,
}

// Tokenizer function to convert expression string into tokens
fn tokenize(expression: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = expression.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Skip whitespace
            ' ' | '\t' | '\n' | '\r' => continue,

            // Arithmetic operators
            '+' => tokens.push(Token::Plus),
            '-' => {
                // Check if this is a negative sign or subtraction operator
                if tokens.is_empty()
                    || matches!(
                        tokens.last().unwrap(),
                        Token::LeftParen
                            | Token::Plus
                            | Token::Minus
                            | Token::Multiply
                            | Token::Divide
                            | Token::Power
                            | Token::Equals
                    )
                {
                    // This is a negative sign
                    tokens.push(Token::Number(-1.0));
                    tokens.push(Token::Multiply);
                } else {
                    // This is a subtraction operator
                    tokens.push(Token::Minus);
                }
            }
            '*' => tokens.push(Token::Multiply),
            '/' => tokens.push(Token::Divide),
            '^' => tokens.push(Token::Power),

            // Parentheses
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),

            // Equals operator for assignment
            '=' => tokens.push(Token::Equals),

            // Numbers
            '0'..='9' | '.' => {
                let mut num_str = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() || next_c == '.' {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match num_str.parse::<f64>() {
                    Ok(num) => tokens.push(Token::Number(num)),
                    Err(_) => return Err(format!("Invalid number: {}", num_str)),
                }
            }

            // Constants, functions, and variables (letters)
            'a'..='z' | 'A'..='Z' => {
                let mut name = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_alphabetic() {
                        name.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Check if it's a constant or function, otherwise treat as variable
                let lowercase_name = name.to_lowercase();
                match lowercase_name.as_str() {
                    "pi" | "e" => tokens.push(Token::Constant(lowercase_name)),
                    "sin" | "exp" => tokens.push(Token::Function(lowercase_name)),
                    _ => tokens.push(Token::Variable(name)),
                }
            }

            // Invalid character
            _ => return Err(format!("Invalid character: {}", c)),
        }
    }

    Ok(tokens)
}

// Parser struct to hold token iterator and variable map
struct Parser<'a> {
    tokens: std::iter::Peekable<std::slice::Iter<'a, Token>>,
    variables: &'a mut std::collections::HashMap<String, f64>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token], variables: &'a mut std::collections::HashMap<String, f64>) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
            variables,
        }
    }

    // Parse a primary expression (number, constant, function call, or parenthesized expression)
    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.tokens.next() {
            Some(Token::Number(num)) => Ok(*num),

            Some(Token::Constant(name)) => match name.as_str() {
                "pi" => Ok(PI),
                "e" => Ok(E),
                _ => Err(format!("Unknown constant: {}", name)),
            },

            Some(Token::Function(name)) => {
                // Parse function call: function_name(expression)
                if let Some(Token::LeftParen) = self.tokens.next() {
                    let arg = self.parse_expression()?;
                    if let Some(Token::RightParen) = self.tokens.next() {
                        match name.as_str() {
                            "sin" => Ok(arg.sin()),
                            "exp" => Ok(arg.exp()),
                            _ => Err(format!("Unknown function: {}", name)),
                        }
                    } else {
                        Err("Missing closing parenthesis".to_string())
                    }
                } else {
                    Err("Missing opening parenthesis after function name".to_string())
                }
            }

            Some(Token::LeftParen) => {
                let expr = self.parse_expression()?;
                if let Some(Token::RightParen) = self.tokens.next() {
                    Ok(expr)
                } else {
                    Err("Missing closing parenthesis".to_string())
                }
            }

            Some(Token::Variable(name)) => {
                // Look up variable in the variables map
                if let Some(&value) = self.variables.get(name) {
                    Ok(value)
                } else {
                    Err(format!("Unknown variable: {}", name))
                }
            }

            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            None => Err("Unexpected end of expression".to_string()),
        }
    }

    // Parse exponentiation (right-associative)
    fn parse_exponent(&mut self) -> Result<f64, String> {
        let mut left = self.parse_primary()?;

        while let Some(Token::Power) = self.tokens.peek() {
            self.tokens.next(); // Consume the ^ token
            let right = self.parse_primary()?;
            left = left.powf(right);
        }

        Ok(left)
    }

    // Parse multiplication and division
    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_exponent()?;

        loop {
            match self.tokens.peek() {
                Some(Token::Multiply) => {
                    self.tokens.next(); // Consume the * token
                    let right = self.parse_exponent()?;
                    left *= right;
                }
                Some(Token::Divide) => {
                    self.tokens.next(); // Consume the / token
                    let right = self.parse_exponent()?;
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left /= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // Parse assignment expressions: variable = expression
    fn parse_assignment(&mut self) -> Result<f64, String> {
        // First check if we have an assignment
        if let Some(Token::Variable(name)) = self.tokens.peek() {
            // Look ahead to see if there's an equals sign
            let mut peek_iter = self.tokens.clone();
            peek_iter.next(); // Skip the variable
            if let Some(Token::Equals) = peek_iter.peek() {
                // It's an assignment: consume the variable and equals sign
                self.tokens.next(); // Consume the variable
                self.tokens.next(); // Consume the equals sign

                // Parse the expression on the right-hand side
                let value = self.parse_expression()?;

                // Update the variable in the map
                self.variables.insert(name.clone(), value);

                // Return the assigned value
                return Ok(value);
            }
        }

        // Not an assignment, parse as regular expression
        self.parse_expression()
    }

    // Parse addition and subtraction
    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;

        loop {
            match self.tokens.peek() {
                Some(Token::Plus) => {
                    self.tokens.next(); // Consume the + token
                    let right = self.parse_term()?;
                    left += right;
                }
                Some(Token::Minus) => {
                    self.tokens.next(); // Consume the - token
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }
}

// Expression evaluation function with support for advanced features
pub fn evaluate_expression(
    expression: &str,
    variables: &mut std::collections::HashMap<String, f64>,
) -> Result<f64, String> {
    // Trim whitespace from the expression
    let trimmed = expression.trim();

    if trimmed.is_empty() {
        return Err("Empty expression".to_string());
    }

    // Tokenize the expression
    let tokens = tokenize(trimmed)?;

    // Parse and evaluate the expression
    let mut parser = Parser::new(&tokens, variables);
    parser.parse_assignment()
}
