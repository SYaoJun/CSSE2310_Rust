#!/bin/sh
# Usage: run_uqentropy_and_save_first_three_lines.sh [args]
# Runs uqentropy with the given arguments
# Only the first three lines from standard output will be saved (output
# to standard output)

# Run the program and only output the first 2 lines. Redirect stderr to /dev/null
${uqentropy:=./uqentropy} "$@" 2>/dev/null | head -3
# There is a possibility uqentropy gets a SIGPIPE. We just exit
# with exit status 0 here.
exit 0

