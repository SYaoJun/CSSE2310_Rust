# Project Overview

## Description

The UQEntropy project is a password strength checker tool written in Rust. It evaluates password strength using entropy calculations and provides additional security analysis by comparing candidate passwords against dictionaries with various transformations.

## Key Features

1. **Entropy Calculation**: Calculates password entropy based on character set size and password length
2. **Password Strength Rating**: Classifies passwords into "very weak", "weak", "strong", or "very strong" categories
3. **Dictionary Matching**: Checks passwords against provided dictionary files
4. **Advanced Transformations**:
   - Case-insensitive matching (`--case`)
   - Digit appending (`--digit-append`)
   - Double word combinations (`--double`)
   - Leet-speak transformations (`--leet`)
5. **Logging**: Comprehensive logging for debugging and audit purposes

## Files Structure

```
.
├── src/
│   └── bin/
│       └── uqentropy.rs  # Main application source code
├── testfiles/            # Test data and scripts
├── Cargo.toml           # Rust package configuration
├── Makefile             # Build automation
├── README.md            # Project documentation
└── testa1_rust.sh       # Test script
```

## Technical Details

- Language: Rust
- Dependencies: log, env_logger, chrono
- Build system: Cargo
- Testing: Custom bash script with predefined test cases