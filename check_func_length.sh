#!/bin/bash

# Check if a file argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <rust_source_file>"
    exit 1
fi

# Check if the file exists
if [ ! -f "$1" ]; then
    echo "Error: File '$1' not found"
    exit 1
fi

# Use sed to extract function definitions and awk to check their lengths
# This is a simple approach that works for basic cases
awk '
BEGIN { 
    in_function = 0
    line_count = 0
}

/^[[:space:]]*fn[[:space:]]+[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]*\(/ {
    if (in_function) {
        # This shouldn't happen, but just in case
        if (line_count > 50) {
            print "Function ending at line " NR " has " line_count " lines"
        }
    }
    in_function = 1
    line_count = 1
    function_name = $0
    gsub(/^[[:space:]]*fn[[:space:]]+/, "", function_name)
    gsub(/[[:space:]]*\(.*$/, "", function_name)
    next
}

in_function {
    line_count++
    if ($0 ~ /^[[:space:]]*\}/ && $0 !~ /.*\{.*\}.*/) {
        # End of function (simple detection)
        in_function = 0
        if (line_count > 50) {
            print "Function " function_name " has " line_count " lines"
        }
    }
}

END {
    # Check if we're still in a function (in case of malformed code)
    if (in_function && line_count > 50) {
        print "Function " function_name " has " line_count " lines (possibly incomplete)"
    }
}
' "$1"