#!/bin/bash

# Usage: run_uqcshell.sh </path/to/program> <delay-seconds> [program-args...]
# Spawns a child program with its stdin/stdout/stderr connected via pipes.
# Reads lines from this script's stdin and forwards to the child's stdin.
# After each forwarded line, sleeps for a delay.
# After sleeping, non-blockingly polls child's stdout/stderr and relays them.
# If the child program exits, drains remaining outputs and exits with the child's exit status.

# Used for mapping PIDs to process numbers
declare -a PIDs=()

# Treat references to unset variables as an error (fail fast).
set -u

# Ignore SIGPIPE 
trap '' PIPE

# ---------- Argument parsing & validation ----------

# If fewer than 2 arguments, print usage to stderr and exit.
if (( $# < 2 )); then
    echo "Usage: $0 <delay-seconds> <exit-method> [program-args...]" >&2
    exit 99
fi

# First positional arg is the program to run.
prog=${uqcshell:=./uqcshell}
# Second positional arg is the sleep delay after forwarding each line.
delay=$1
# Third positional arg is the exit method. exitTerm will send SIGTERM
# and exitEOF will close stdin pipe to uqcshell
exitMethod=$2
# Shift away the second arg so $@ now contains only the child's arguments.
shift 2
# Capture remaining program args into an array to preserve spaces/quoting.
progArgs=( "$@" )

# Validate that the delay looks like a number (simple check: digits and at most one dot).
# This avoids passing junk (like 'abc') to sleep.
# Accept N, N., .N, or N.N (at least one digit somewhere, max one dot)
if ! [[ $delay =~ ^([0-9]+(\.[0-9]*)?|\.[0-9]+)$ ]]; then
    echo "Error: delay must be a non-negative number (e.g., 0.5, 1, 2.0), got '$delay'." >&2
    exit 98
fi

# Check that the program exists (either absolute path or resolvable via PATH).
if ! command -v -- "$prog" >/dev/null 2>&1; then
    echo "Error: program '$prog' not found in PATH or not executable." >&2
    exit 97
fi
# ---------- Setup FIFOs ----------

# Create a dedicated temporary directory for our FIFOs (named pipes).
tmpdir=$(mktemp -d -t uqcshell.XXXXXXXX) || {
    echo "Error: failed to create temp dir." >&2
    exit 96
}

# Define FIFO paths:
# inFIFO: our writing end -> becomes child's stdin
# outFIFO: child's stdout -> our reading end
# errFIFO: child's stderr -> our reading end
inFIFO="$tmpdir/in.fifo"
outFIFO="$tmpdir/out.fifo"
errFIFO="$tmpdir/err.fifo"

# Define a cleanup function to run on exit or interruption.
cleanup() {
    # Close any of our custom file descriptors if open (fd 3, 4, 5).
    exec 3>&- 4<&- 5<&- 2>/dev/null || true
    # If the child is still running, request it to stop.
    if [[ -n "${childPID:-}" ]] && kill -0 "$childPID" 2>/dev/null; then
        # First, try a gentle kill (SIGTERM).
        kill "$childPID" 2>/dev/null || true
        # Give it a brief moment to exit.
        sleep 1.0
        # If still alive, force kill (SIGKILL).
        kill -9 "$childPID" 2>/dev/null || true
    fi
    # Remove FIFO files; ignore errors if they're already gone.
    rm -f -- "$inFIFO" "$outFIFO" "$errFIFO" 2>/dev/null || true
    # Remove the temporary directory; ignore errors if it's already gone.
    rmdir -- "$tmpdir" 2>/dev/null || true
}

# Ensure cleanup runs when the script exits (normal or due to a signal).
trap cleanup EXIT

# Create the three named pipes (FIFOs).
mkfifo "$inFIFO" "$outFIFO" "$errFIFO" || {
    echo "Error: mkfifo failed." >&2
    exit 95
}

# ---------- Launch child process ----------

# Use stdbuf to set line-buffered stdout/stderr for the child.
stdbuf -oL -eL -- "$prog" "${progArgs[@]}" <"$inFIFO" >"$outFIFO" 2>"$errFIFO" &

# Capture the child PID.
childPID=$!

# Wait briefly for child to initialise.
sleep 0.1
if ! kill -0 "$childPID" 2>/dev/null; then
    echo "Error: child process failed to start" >&2
    exit 94
fi

# ---------- Open our ends of the FIFOs ----------

# Open the write end of inFIFO on file descriptor 3 (we write to child's stdin via fd 3).
exec 3>"$inFIFO"
# Open the read end of outFIFO on file descriptor 4 (we read child's stdout via fd 4).
exec 4<"$outFIFO"
# Open the read end of errFIFO on file descriptor 5 (we read child's stderr via fd 5).
exec 5<"$errFIFO"

# Define a helper to forward a specific signal to the child (if still alive).
forward_sig() { 
    kill -s "$1" "$childPID" 2>/dev/null || true; 
}

# Forward Ctrl-C (SIGINT) from the wrapper to the child.
trap 'forward_sig INT' INT
# Forward SIGTERM from the wrapper to the child as well.
trap 'forward_sig TERM' TERM

# ---------- Helpers ----------

# Processes a string to find "pattern y" where
# y is an integer PID. Converts the PID value to an array index.
# Outputs the modified line where PID values are replaced with indices.
# If PID exists in PIDs array: replace y with its array index.
# If PID doesn't exist: add y to PIDs array and use new index.
# If no PID pattern found: return original line unchanged.
map_pid() {
    local line="$1"    # Input line to process
    local outVar=$2    # Variable to receive the output line
    local pattern=$3   # Pattern to search for
    local rest="$line" # Portion of the line we still need to scan/process
    local out=""       # Accumulated output (rewritten) as we progress
    # Temporaries used per match:
    local match        # The exact matched text (e.g., "PID   123")
    local whiteSpace   # The whitespace captured after "PID"
    local pidValue     # The numeric PID string captured (e.g., "123")
    local foundIndex   # The index assigned to pidValue in PIDs
    local head tail    # Substrings before/after the first occurrence of 'match'
    local repl         # Replacement text for the current match (e.g., "PID   4")
    # Process all occurrences of '<pattern> <spaces> <digits>' in the line.
    while [[ $rest =~ "$pattern"([[:space:]]+)([0-9]+) ]]; do
        match=${BASH_REMATCH[0]} # Entire match
        whiteSpace=${BASH_REMATCH[1]} # Captured whitespace after pattern
        pidValue=${BASH_REMATCH[2]} # Captured digits (PID number)
        # Find or assign the index for this PID in the global PIDs array.
        foundIndex=-1
        for i in "${!PIDs[@]}"; do
            if [[ ${PIDs[i]} == "$pidValue" ]]; then
                foundIndex=$i
                break
            fi
        done
        if (( foundIndex < 0 )); then
            PIDs+=("$pidValue")              # First time we've seen this PID so append
            foundIndex=$((${#PIDs[@]} - 1))  # Its index is the last position
        fi
        repl="${pattern}${whiteSpace}${foundIndex}" # Build the replacement while preserving whitespace
        # Split 'rest' around the first occurrence of 'match'.
        head=${rest%%"$match"*} # Everything before match
        tail=${rest#*"$match"} # Everything after match
        # Append rewritten chunk to output and continue scanning the remainder.
        out+="$head$repl"
        rest="$tail"
    done
    # Append any leftover text that had no matches.
    out+="$rest"
    # Write the final transformed line into the caller-provided variable name.
    printf -v "$outVar" '%s' "$out"
}

# Drain a file descriptor's output without blocking. 
# The tiny timeout lets us peek at readiness; if nothing is available, we move on.
# $1: The file descriptor number to drain (e.g., 4 for stdout, 5 for stderr).
# $2: The output file descriptor to write to (e.g., 1 for stdout, 2 for stderr).
drain_fd() {
    local str
    local fdIn=$1
    local fdOut=$2
    local modifiedStr
    local modifiedStr2
    while true; do
        # Attempt to read a chunk of data. read returns a non-zero exit code
        # on timeout, but still populates the variable 'str'.
        read -r -d '' -t 0.1 str <&"$fdIn"
        # Attempt to read a chunk of data. read returns a non-zero exit code
        # on timeout, but still populates the variable 'str'.
        if [[ -n "$str" ]]; then
            map_pid "$str" modifiedStr "PID"  # Format for most commands
            map_pid "$modifiedStr" modifiedStr2 "," # Format for list command
            # Print the partial line. We add a newline to ensure it gets flushed
            # from the buffer and becomes visible immediately.
            printf '%s\n' "$modifiedStr2" >&"$fdOut"
        else
            # Exit the loop if nothing is read, as the buffer is empty.
            break
        fi
    done
}

# Attempt to drain anything currently available from child's stdout and stderr
# without blocking.
drain_outputs_once() {
    # Drain stderr (fd 5) and write to stderr (fd 2)
    drain_fd 5 2
    # Drain stdout (fd 4) and write to stdout (fd 1)
    drain_fd 4 1
}

# Return success if the child process still exists; otherwise failure.
child_is_alive() {
    kill -0 "$childPID" 2>/dev/null
}

# When the child is done (or its stdin is closed), fully drain remaining output
# (blocking reads until EOF on both stdout/stderr) and exit with child's status.
finish_and_exit_with_child_status() {
    local line
    # Drain child stdout until EOF (fd 4).
    while IFS= read -r line <&4; do
        printf '%s\n' "$line"
    done
    # Drain child stderr until EOF (fd 5).
    while IFS= read -r line <&5; do
        printf '%s\n' "$line" >&2
    done
    # Wait for the child and capture its exit status.
    local status=0
    wait "$childPID"
    status=$?
    # If the child died due to a signal, bash encodes it as 128+signal.
    if [ "$status" -ge 128 ] && [ "$status" -le 255 ]; then
        sig=$(( status - 128 ))
        exit "$sig"
    fi
    # Exit this wrapper with the exact same status code as the child.
    exit "$status"
}

# Replaces tokens like <0>, <12>, ... in a single input line with the corresponding element from the global array PIDs.
# Leaves tokens unchanged if the index is out of bounds or unset.
# Does not perform recursive expansion of results.
expand_placeholders() {
    local line="$1"
    # out accumulates the result, idx holds the numeric index from <idx>
    local out="" idx
    # Loop while the line still contains a <number> pattern.
    while [[ $line =~ ^([^<]*)\<([0-9]+)\>(.*)$ ]]; do
        # Append text before the token to the output
        out+="${BASH_REMATCH[1]}"
        # Extract the numeric index inside <...>
        idx="${BASH_REMATCH[2]}"
        # If PIDs[idx] is set (even if empty), substitute it.
        if [[ -n ${PIDs[$idx]+_} ]]; then
            out+="${PIDs[$idx]}"
        else
            # If the index is out of range or unset, keep the original token untouched
            out+="<${idx}>"
        fi
        # Continue processing the remainder of the line after the matched token
        line="${BASH_REMATCH[3]}"
    done
    # Append whatever remains (no more <n> patterns found)
    out+="$line"
    # Print the final expanded line
    printf '%s\n' "$out"
}

# ---------- Main loop: read from our stdin, forward to child, sleep, poll outputs ----------

if ! child_is_alive; then
    finish_and_exit_with_child_status
fi
sleep "$delay"
# Check for initial print statements by the program.
drain_outputs_once
# Read one line at a time from this wrapper's stdin.
while IFS= read -r line; do
    # If the child already died, stop reading input and finish immediately.
    if ! child_is_alive; then
        finish_and_exit_with_child_status
    fi
    if [[ "$line" == "<SIGINT>" ]]; then
        kill -SIGINT "$childPID" 2>/dev/null
        sleep "$delay"
        continue
    fi
    modifiedLine="$(expand_placeholders "$line")"
    # Forward the line to the child's stdin via fd 3, followed by a newline.
    if ! printf '%s\n' "$modifiedLine" >&3; then
        # If writing fails (child closed stdin), finish now.
        finish_and_exit_with_child_status
    fi
    # Sleep for the configured delay after forwarding each line.
    sleep "$delay"
    # Non-blockingly poll any available outputs and relay them.
    drain_outputs_once
    # Check if the child exited during/after the above steps; if so, finish.
    if ! child_is_alive; then
        finish_and_exit_with_child_status
    fi
done

if [ "$exitMethod" == "exitEOF" ]; then
    # If our stdin hit EOF (no more lines), close the child's stdin to signal completion.
    exec 3>&-    
elif [ "$exitMethod" == "exitTerm" ]; then
    kill -15 "$childPID" 2>/dev/null
else
    kill -15 "$childPID" 2>/dev/null
fi

# After signaling EOF to the child, continue polling until it actually exits.
while child_is_alive; do
    drain_outputs_once
    sleep "$delay"
done

# Child has exited: drain any remaining buffered output to completion and exit with its status.
finish_and_exit_with_child_status
