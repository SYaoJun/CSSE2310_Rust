use a1_2025_s1::*;
use std::process::exit;

// Function to format results according to test expectations
fn format_result(result: f64, sigfigs: u8) -> String {
    // Zero is always "0"
    if result == 0.0 {
        return "0".to_string();
    }

    let abs = result.abs();

    // Decide between fixed and scientific notation similarly to tests:
    // use scientific for very large/small magnitudes (always apply this rule)
    let use_sci = abs >= 1e4 || abs < 1e-3;

    if use_sci {
        // scientific notation with required significant figures.
        // Rust's {:.Ne} uses N digits after decimal, i.e. total sig figs = N+1
        let decimals = if sigfigs > 0 { sigfigs - 1 } else { 0 };
        let formatted = format!("{:.*e}", decimals as usize, result);

        // Normalise exponent to always have sign and at least two digits.
        if let Some(idx) = formatted.find('e') {
            let (mantissa, exp_part) = formatted.split_at(idx);
            let exp_num: i32 = exp_part[1..].parse().unwrap_or(0);
            return format!("{}e{:+03}", mantissa, exp_num);
        }
        return formatted;
    }

    // Fixed notation: if sigfigs is 0, show as plain number
    if sigfigs == 0 {
        if result.fract() == 0.0 {
            return format!("{}", result as i64);
        }
        return format!("{}", result);
    }

    // Fixed notation with significant figures rounding
    let digits_before = abs.log10().floor() as i32 + 1;
    let decimals = (sigfigs as i32 - digits_before).max(0) as usize;
    let rounded = format!("{:.*}", decimals, result);
    rounded
}

// Function to format variable and loop variable values for display
fn format_var(value: f64) -> String {
    // Zero is always "0"
    if value == 0.0 {
        return "0".to_string();
    }

    let abs = value.abs();

    // Use scientific notation for very large/small numbers
    if abs >= 1e4 || abs < 1e-3 {
        // Format with 1 significant figure for large/small numbers
        let formatted = format!("{:.0e}", value);
        // Normalise exponent to have sign and at least two digits
        if let Some(idx) = formatted.find('e') {
            let (mantissa, exp_part) = formatted.split_at(idx);
            let exp_num: i32 = exp_part[1..].parse().unwrap_or(0);
            return format!("{}e{:+03}", mantissa, exp_num);
        }
        return formatted;
    }

    // For normal numbers, format to 4 significant figures
    // Handle different cases based on magnitude
    let digits_before = abs.log10().floor() as i32 + 1;
    let decimals = (4 - digits_before).max(0) as usize;
    format!("{:.*}", decimals, value)
}

fn main() {
    let res = handle_command_line_arguments();
    if res.is_err() {
        eprintln!("{}", USAGE_MSG);
        exit(ExitCodes::Usage as i32);
    }
    
    let mut config = res.unwrap();
    
    // 检查文件是否存在，如果存在且不可读，则在输出欢迎信息前退出
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
    
    // Check for variables
    let res = check_variable(&mut config);
    match res {
        Err(ExitError::Variable) => {
            eprintln!("{}", VARIABLE_MSG);
            exit(ExitCodes::Variable as i32);
        }
        Err(ExitError::Duplicate) => {
            eprintln!("{}", DUPLICATE_MSG);
            exit(ExitCodes::Duplicate as i32);
        }
        _ => {
            // Print welcome message
            println!("Welcome to uqexpr.");
            println!("This program was written by @yaojun.");

            // Print variable information
            if config.init_map.is_empty() {
                println!("No variables were identified.");
            } else {
                println!("Variables:");
                // Variables are printed in lexicographic order by name to match tests.
                let mut names: Vec<_> = config.init_map.keys().cloned().collect();
                names.sort();
                for name in names {
                    if let Some(value) = config.init_map.get(&name) {
                        let formatted_value = format_var(*value);
                        println!("{} = {}", name, formatted_value);
                    }
                }
            }
            
            // Print loop information
            if config.for_loop_struct_vec.is_empty() {
                println!("There are no loop variables.");
            } else {
                println!("Loop variables:");
                for loop_var in &config.for_loop_struct_vec {
                    // Format increment using the same logic as format_result for consistency
                    let increment = loop_var.increment;
                    let increment_formatted = if increment.abs() >= 1e4 || increment.abs() < 1e-3 {
                        // Use scientific notation for large/small numbers
                        let decimals = 0; // 1 significant figure after decimal for e notation
                        let formatted = format!("{:.*e}", decimals, increment);
                        // Normalise exponent to have sign and at least two digits
                        if let Some(idx) = formatted.find('e') {
                            let (mantissa, exp_part) = formatted.split_at(idx);
                            let exp_num: i32 = exp_part[1..].parse().unwrap_or(0);
                            format!("{}e{:+03}", mantissa, exp_num)
                        } else {
                            formatted
                        }
                    } else if increment.fract() == 0.0 {
                        // Integer format for whole numbers in normal range
                        format!("{}", increment as i64)
                    } else {
                        // Default string format for others
                        format!("{}", increment)
                    };
                    
                    // Format all loop values using format_var
                    let start_formatted = format_var(loop_var.start);
                    let end_formatted = format_var(loop_var.end);
                    // Print in correct order: name = start (start, end, increment)
                    println!("{} = {} ({}, {}, {})", loop_var.name, start_formatted, start_formatted, end_formatted, increment_formatted);
                }
            }
            
            // Handle file input if provided
            if config.filename_flag {
                // No prompt needed for file input
            } else {
                // Print prompt for interactive input
                println!("Please enter your expressions and assignment operations to be evaluated.");
            }
            
            // Handle file input if provided
            if config.filename_flag {
                let file_string_res = check_input_filename(&config.input_filename);
                if file_string_res.is_err() {
                    eprintln!(
                        "uqexpr: unable to read from input file \"{}\"",
                        config.input_filename
                    );
                    exit(ExitCodes::InvalidFile as i32);
                }
                // Process file content
                for line in file_string_res.unwrap() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Check if it's an assignment
                            if trimmed.contains('=') {
                                match evaluate_expression(&trimmed, &mut config.init_map) {
                                    Ok(result) => {
                                        let formatted_result =
                                            format_result(result, config.significant_figures);
                                        let parts: Vec<&str> = trimmed.split('=').collect();
                                        let var_name = parts[0].trim();
                                        println!("{} = {}", var_name, formatted_result);
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "Invalid command, expression or assignment operation detected"
                                        );
                                    }
                                }
                            } else {
                                match evaluate_expression(&trimmed, &mut config.init_map) {
                                    Ok(result) => {
                                        let formatted_result =
                                            format_result(result, config.significant_figures);
                                        println!("Result = {}", formatted_result);
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "Invalid command, expression or assignment operation detected"
                                        );
                                    }
                                }
                            }
                    }
                }
            } else {
                // Read from standard input
                use std::io::{self, BufRead};
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    match line {
                        Ok(expression) => {
                            let trimmed = expression.trim();
                            if !trimmed.is_empty() {
                                if trimmed.starts_with("@print") {
                                    // @print command: print variable value without "Result ="
                                    let var_name =
                                        trimmed.trim_start_matches("@print").trim();
                                    if let Some(value) = config.init_map.get(var_name) {
                                        let formatted =
                                            format_result(*value, config.significant_figures);
                                        println!("{}", formatted);
                                    } else {
                                        eprintln!(
                                            "Invalid command, expression or assignment operation detected"
                                        );
                                    }
                                } else if trimmed.contains('=') {
                                    match evaluate_expression(&trimmed, &mut config.init_map) {
                                        Ok(result) => {
                                            let formatted_result =
                                                format_result(result, config.significant_figures);
                                            let parts: Vec<&str> =
                                                trimmed.split('=').collect();
                                            let var_name = parts[0].trim();
                                            println!("{} = {}", var_name, formatted_result);
                                        }
                                        Err(_) => {
                                            eprintln!(
                                                "Invalid command, expression or assignment operation detected"
                                            );
                                        }
                                    }
                                } else {
                                    match evaluate_expression(&trimmed, &mut config.init_map) {
                                        Ok(result) => {
                                            let formatted_result =
                                                format_result(result, config.significant_figures);
                                            println!("Result = {}", formatted_result);
                                        }
                                        Err(_) => {
                                            eprintln!(
                                                "Invalid command, expression or assignment operation detected"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error reading input: {}", err);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // Print closing message
    println!("Thank you for using uqexpr.");
}