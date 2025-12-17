#!/bin/bash
# Usage: run_client_with_valgrind.sh args
LD_PRELOAD=

PATH=${PATH}:/bin:/usr/bin

# Remove temporary file
rm -f /tmp/csse2310.valgrind.$$.out

# Run client within valgrind 
/usr/bin/valgrind --log-file=/tmp/csse2310.valgrind.$$.out ${ratsclient:=./ratsclient} "$@"
status=$?

# Check for memory leaks in the valgrind log
msg="All heap blocks were freed"
if grep "$msg" /tmp/csse2310.valgrind.$$.out &>/dev/null ; then
    echo "$msg" >&2
else
    echo "Memory leak found" >&2
fi

# Remove valgrind output file
rm -f /tmp/csse2310.valgrind.$$.out

exit $status
