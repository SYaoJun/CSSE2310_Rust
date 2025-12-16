#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize total score variables
total_score=0
total_possible=0
verbose=0
use_valgrind=0
explain_mode=0

# Trap Ctrl+C (SIGINT) to cleanly exit
trap 'echo -e "\n${YELLOW}Test execution interrupted by user${NC}"; exit 130' INT

# Function to show usage
show_usage() {
    echo "Usage: $0 [options] [run <test_number> ...] [explain <test_number>]"
    echo "Options:"
    echo "  -v, --verbose    Show detailed output for failed tests using vimdiff"
    echo "  -h, --help       Show this help message"
    echo "  --valgrind       Run memory leak tests with Valgrind"
    echo "Commands:"
    echo "  run <test> ...   Run specific test(s) or test category(ies)"
    echo "                   Examples: run 2       (run all 2.* tests)"
    echo "                            run 2.1      (run test 2.1)"
    echo "                            run 2 3 4    (run all 2.*, 3.*, 4.* tests)"
    echo "  explain <test>   Show detailed explanation of a test"
    exit 1
}


# Parse command line options
test_patterns=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--verbose)
            verbose=1
            shift
            ;;
        -h|--help)
            show_usage
            ;;
        --valgrind)
            use_valgrind=1
            shift
            ;;
        run)
            shift
            # Collect all test patterns until we hit another option or end
            while [[ $# -gt 0 ]] && [[ ! "$1" =~ ^- ]]; do
                test_patterns+=("$1")
                shift
            done
            if [ ${#test_patterns[@]} -eq 0 ]; then
                echo "Error: 'run' command requires at least one test number"
                show_usage
            fi
            ;;
        explain)
            if [[ $# -lt 2 ]]; then
                show_usage
            fi
            explain_mode=1
            test_pattern="$2"
            shift 2
            ;;
        *)
            show_usage
            ;;
    esac
done

# Check if Valgrind is installed when needed
if [ $use_valgrind -eq 1 ]; then
    if ! command -v valgrind &> /dev/null; then
        echo -e "${YELLOW}Warning: Valgrind is not installed. Memory leak tests will be skipped.${NC}"
        echo "To install Valgrind:"
        echo "  Ubuntu/Debian: sudo apt-get install valgrind"
        echo "  macOS: brew install valgrind"
        exit 1
    fi
fi

# Function to show diff using vimdiff
show_diff() {
    local actual=$1
    local expected=$2
    local type=$3
    
    # Create temporary files for vimdiff
    local actual_tmp=$(mktemp)
    local expected_tmp=$(mktemp)
    
    # Copy contents to temporary files
    cat "$actual" > "$actual_tmp"
    cat "$expected" > "$expected_tmp"
    
    # Show diff using vimdiff
    echo "Press 'q' to quit vimdiff"
    echo "Showing $type differences..."
    sleep 1
    vimdiff "$expected_tmp" "$actual_tmp"
    
    # Clean up temporary files
    rm -f "$actual_tmp" "$expected_tmp"
}

# Compile the program
cargo build --bin uqentropy &> /dev/null
if [ $? -ne 0 ]; then
    echo "Compilation failed"
    exit 1
fi

cp ../target/debug/uqentropy .
# 拷贝是否成功判断
if [ $? -ne 0 ]; then
    echo "Failed to copy uqentropy executable"
    exit 1
fi
make &> /dev/null

# Function to run a test
run_test() {
    local test_number=$1
    local test_name=$2
    local description=$3
    shift 3
    
    # If in explain mode, show test details and return
    if [ $explain_mode -eq 1 ]; then
        explain_test "$@"
        return
    fi
    
    # Process options
    local timeout=2
    local stdin="/dev/null"
    local expected_exit=0
    local expected_stdout="testfiles/empty"
    local expected_stderr="testfiles/empty"
    local expected_file=""
    local expected_file_path=""
    local actual_file_path=""
    local cmd=()
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --timeout | -timeout)
                timeout="$2"
                shift 2
                ;;
            --stdin | -stdin)
                stdin="$2"
                shift 2
                ;;
            --exit | -exit)
                expected_exit="$2"
                shift 2
                ;;
            --stdout | -stdout)
                expected_stdout="$2"
                shift 2
                ;;
            --stderr|-stderr)
                expected_stderr="$2"
                shift 2
                ;;
            --optional)
                shift 1
                ;;
            --maxattempts|-maxattempts)
                shift 2
                ;;
            --file|-file)
                actual_file_path="$2"
                expected_file="$3"
                shift 3
                ;;
            *)
                cmd+=("$1")
                shift
                ;;
        esac
    done
    
    # Calculate max score based on test category
    local max_score=0
    case "${test_number%.*}" in
        1) max_score=0.5000 ;;
        2) max_score=0.4000 ;;
        3) max_score=0.3333 ;;
        4) max_score=0.7500 ;;
        5) max_score=1.0000 ;;
        6) max_score=0.7500 ;;
        7) max_score=1.0000 ;;
        8) max_score=0.5000 ;;
        9) max_score=0.5000 ;;
        10) max_score=0.5000 ;;
        11) max_score=1.0000 ;;
        12) max_score=1.0000 ;;
        13) max_score=1.0000 ;;
        14) max_score=1.0000 ;;
        15) max_score=0.5000 ;;
        16) max_score=1.0000 ;;
        17) max_score=0.5000 ;;

    esac
    total_possible=$(echo "$total_possible + $max_score" | bc)
    
    # Create temporary files for output
    stdout_file=$(mktemp)
    stderr_file=$(mktemp)
    
    # Run command with timeout
    if [ ${#cmd[@]} -eq 0 ]; then
        ./program < "$stdin" > "$stdout_file" 2> "$stderr_file"
    else
        "${cmd[@]}" < "$stdin" > "$stdout_file" 2> "$stderr_file"
    fi
    exit_code=$?
    
    # Compare outputs
    stdout_diff=0
    stderr_diff=0
    file_diff=0
    
    if [ -f "$expected_stdout" ]; then
        diff -w "$stdout_file" "$expected_stdout" > /dev/null
        stdout_diff=$?
    fi
    
    if [ -f "$expected_stderr" ]; then
        diff -w "$stderr_file" "$expected_stderr" > /dev/null
        stderr_diff=$?
    fi

    # Check file content if --file parameter was provided
    if [ -n "$expected_file" ] && [ -n "$actual_file_path" ]; then
        if [ -f "$actual_file_path" ] && [ -f "$expected_file" ]; then
            diff -w "$actual_file_path" "$expected_file" > /dev/null
            file_diff=$?
        else
            file_diff=1
            if [ ! -f "$actual_file_path" ]; then
                echo "    Output file not found: $actual_file_path" >&2
            fi
            if [ ! -f "$expected_file" ]; then
                echo "    Expected file not found: $expected_file" >&2
            fi
        fi
    fi
    
    # Check results
    if [ "$exit_code" -eq "$expected_exit" ] && [ $stdout_diff -eq 0 ] && [ $stderr_diff -eq 0 ] && [ $file_diff -eq 0 ]; then
        printf "%s - %-33s ${GREEN}PASS${NC}    %5.4f / %5.4f\n" "$test_number" "$test_name" "$max_score" "$max_score"
        total_score=$(echo "$total_score + $max_score" | bc)
    else
        printf "%s - %-33s ${RED}FAIL${NC}    %5.4f / %5.4f\n" "$test_number" "$test_name" 0 "$max_score"
        if [ $stdout_diff -ne 0 ]; then
            echo "    Mismatch on stdout" >&2
            if [ $verbose -eq 1 ]; then
                show_diff "$stdout_file" "$expected_stdout" "stdout"
            fi
        fi
        if [ $stderr_diff -ne 0 ]; then
            echo "    Mismatch on stderr" >&2
            if [ $verbose -eq 1 ]; then
                show_diff "$stderr_file" "$expected_stderr" "stderr"
            fi
        fi
        if [ $file_diff -ne 0 ] && [ -n "$expected_file" ]; then
            echo "    Mismatch on file: $actual_file_path" >&2
            if [ $verbose -eq 1 ] && [ -f "$actual_file_path" ]; then
                show_diff "$actual_file_path" "$expected_file" "file"
            fi
        fi
        if [ "$exit_code" -ne "$expected_exit" ]; then
            echo "    Expected exit code: $expected_exit" >&2
            echo "    Actual exit code: $exit_code" >&2
        fi
    fi
    
    rm -f "$stdout_file" "$stderr_file"
}

echo "Running tests"
echo "============================================================"


for def in testfiles/.definitions/*.definition; do
    source "$def"
done


run_tests_by_pattern() {
    local pattern=$1

    for major in {1..17}; do  # 包含所有测试类别
        files=$(ls testfiles/.definitions/$major.*.definition 2>/dev/null | sort -V)
        if [ -n "$files" ]; then
            for test_file in $files; do
                test_func=$(basename "$test_file" .definition)

                # 精确匹配逻辑
                if [[ "$pattern" =~ ^[0-9]+$ ]]; then
                    # 只匹配同一大类的，例如 "1" 匹配 "1.1", "1.2" ...
                    if [[ "$test_func" == $pattern.* ]]; then
                        "test${test_func}" run_test
                    fi
                else
                    # 完全匹配，如 "1.2" 只匹配 "1.2"
                    if [[ "$test_func" == "$pattern" ]]; then
                        "test${test_func}" run_test
                    fi
                fi
            done
        fi
    done
}


if [ ${#test_patterns[@]} -eq 0 ]; then
    # Run all tests
    for major in {1..15}; do
        files=$(ls testfiles/.definitions/$major.*.definition 2>/dev/null | sort -V)
        if [ -n "$files" ]; then
            for test_file in $files; do
                test_func=$(basename "$test_file" .definition)
                "test${test_func}" run_test
            done
        fi
    done
else
    # Run tests matching the provided patterns
    for pattern in "${test_patterns[@]}"; do
        run_tests_by_pattern "$pattern"
    done
fi

echo "============================================================"
echo "Tests completed"

if [ $(echo "$total_possible > 0" | bc -l) -eq 1 ]; then
    printf "Total score: %5.4f / %5.4f (%5.2f%%)\n" $total_score $total_possible $(echo "scale=2; $total_score * 100 / $total_possible" | bc)
else
    echo "Total score: 0.0000 / 0.0000 (0.00%)"
fi

make clean