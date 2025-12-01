# Code Analysis

The main application code is located in `src/bin/uqentropy.rs`. Let's examine the key components and their functionalities.

## Main Components

### 1. Configuration Structure

```rust
struct Config {
    leet: bool,
    case_sensitive: bool,
    digit_append: bool,
    double_check: bool,
    num_digits: usize,
}
```

This structure holds the command-line configuration options that control how password matching is performed.

### 2. Entropy Calculation

The entropy calculation is based on the formula: `length * log2(pool_size)` where pool_size depends on the character types present in the password:
- Digits: 10 characters
- Lowercase letters: 26 characters
- Uppercase letters: 26 characters
- Symbols: 32 characters

### 3. Password Strength Mapping

Passwords are categorized based on their entropy values:
- Very weak: < 35.0
- Weak: 35.0 - 59.9
- Strong: 60.0 - 119.9
- Very strong: ≥ 120.0

### 4. Dictionary Matching Algorithms

The application implements several sophisticated matching algorithms:

#### Basic Matching
Direct comparison with dictionary entries.

#### Case-insensitive Matching (`--case`)
Tries all case variations of dictionary words.

#### Digit Appending (`--digit-append`)
Appends sequences of digits (1-8 digits long) to dictionary words.

#### Double Word Combinations (`--double`)
Combines pairs of dictionary words.

#### Leet Speak Transformation (`--leet`)
Applies common character substitutions (e.g., 'a' → '4', 'e' → '3').

### 5. Logging System

The application uses `env_logger` with timestamped log files stored in the `log/` directory for debugging and auditing purposes.

## Key Functions

### `calculate_entropy`
Calculates the entropy of a given password based on character set diversity and length.

### `calculate_entropy_two`
Performs advanced matching against a dictionary with various transformations.

### `read_file` and `read_single_file`
Handle reading and parsing of dictionary files, validating that they contain only printable ASCII characters.

### `process_user_input`
Main interactive loop that reads candidate passwords from standard input and evaluates their strength.

## Error Handling

The application uses custom exit codes:
- 2: Usage errors
- 14: No strong passwords identified
- 20: Invalid file access

This structured approach ensures robust error handling and informative feedback to users.