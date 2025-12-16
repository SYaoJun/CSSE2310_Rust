#!/bin/bash
# Usage: run_uqcshell_with_valgrind.sh args
LD_PRELOAD=

PATH=${PATH}:/bin:/usr/bin

# Remove temporary file
/usr/bin/rm -f /tmp/csse2310.valgrind.$$.out

# Run uqcshell within valgrind 
LD_PRELOAD=testfiles/fork_interposer.so:testfiles/kill_interposer.so /usr/bin/valgrind --trace-children=no --child-silent-after-fork=yes --log-file=/tmp/csse2310.valgrind.$$.out ${uqcshell:=./uqcshell} "$@"
status=$?

# Check for memory leaks in the valgrind log
msg="All heap blocks were freed"
if /usr/bin/grep "$msg" /tmp/csse2310.valgrind.$$.out &>/dev/null ; then
    /usr/bin/echo "$msg" >&2
else
    /usr/bin/echo "Memory leak found" >&2
fi

# Remove valgrind output file
/usr/bin/rm -f /tmp/csse2310.valgrind.$$.out

exit $status
