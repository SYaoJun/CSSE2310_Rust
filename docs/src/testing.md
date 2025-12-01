# Testing

The UQEntropy project includes a comprehensive test suite in the form of a shell script: `testa1_rust.sh`. This script contains multiple test cases organized in categories to thoroughly validate the application's functionality.

## Running Tests

To run the full test suite:

```bash
./testa1_rust.sh
```

To run specific test categories:

```bash
./testa1_rust.sh run 1     # Run all tests in category 1
./testa1_rust.sh run 1.1   # Run only test 1.1
```

To run tests with verbose output:

```bash
./testa1_rust.sh -v run 1
```

## Test Categories

The test suite is organized into the following categories:

1. **Basic functionality tests** - Simple password entropy calculations
2. **Input validation tests** - Handling of valid and invalid inputs
3. **File handling tests** - Processing of dictionary files
4. **Basic matching tests** - Direct password matching against dictionaries
5. **Case sensitivity tests** - Testing the `--case` option
6. **Digit append tests** - Testing the `--digit-append` option
7. **Double word tests** - Testing the `--double` option
8. **Leet speak tests** - Testing the `--leet` option
9. **Combined option tests** - Testing multiple options together
10. **Error condition tests** - Handling of various error scenarios
11. **Special character tests** - Handling of various character sets
12. **Boundary condition tests** - Edge cases and limits
13. **Performance tests** - Efficiency validation
14. **Output format tests** - Correctness of output formatting
15. **Exit code tests** - Proper exit codes for different scenarios

## Test Data

The `testfiles` directory contains various test data files:

- Password dictionaries of different sizes (top10000.txt, top1050.txt, 2023-200_most_used_passwords.txt)
- Scripts for running tests and saving outputs
- Expected output files for validation

## Test Implementation

The test script:
1. Compiles the program
2. Runs each test case with specific inputs and command-line options
3. Captures stdout, stderr, and exit codes
4. Compares actual outputs with expected outputs
5. Reports PASS/FAIL status for each test
6. Calculates overall scores

## Continuous Integration

While not explicitly configured, the test suite could be integrated with CI systems like GitHub Actions to automatically validate changes to the codebase.