#/bin/bash
# wait_for_connection.sh pid port
# Waits for the given pid to have an established network connection to or
# from the given port.
# Exits with status 0 if this happens within 2 seconds, otherwise exits with
# status 1 (timeout)

if [ ! "$1" -o ! "$2" ] ; then
    exit 1
fi
pid="$1"
porthex=$(printf "%04X" "$2")

# Steps
# - file descriptors the process has
# - filter to sockets
# - extract inode
# - look up inode in /proc/self/net/tcp
# - filter to established connections only (state 01)
declare -i count_iterations=0
while [ $count_iterations -lt 20 ] ; do
    if ! ps -p$pid > /dev/null ; then
	# Process does not exist - abort
	exit 1
    fi
    if ls -l /proc/$pid/fd 2>/dev/null | grep socket | cut -d "[" -f 2 | tr -d ] | xargs -I X grep X /proc/net/tcp | grep "${porthex}" | grep " 01 " >& /dev/null ; then
	exit 0
    fi
    sleep 0.1
    # else process exists but does not have an established network connection
    count_iterations+=1
done
exit 1
