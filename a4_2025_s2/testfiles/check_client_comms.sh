#!/bin/bash
# check_client_comms.sh [--debug] [--showclientoutput] \
#       [--serverdisconnect | --showserverrequest ] \
#       [--startfromline n] \
#       [--serverresponse response-file] [client-arguments ...]
# (Client port number argument will be supplied by this script)
PATH=/usr/bin:${PATH}

# Make a copy of stderr and close it (to prevent job control messages)
exec {errfd}>&2
exec 2>&-
# By default, make debugfd point to /dev/null
exec {debugfd}>/dev/null

client=${ratsclient:=./ratsclient}
showclientoutput=0
showserverrequest=0
serverexpectshttp=0
serverdisconnect=0
serverresponse=/dev/null
# "startfromline" applies to the client output and the server request output
startfromline=1

# Process command line arguments
while true ; do
    case "$1" in 
	--debug ) eval "exec ${debugfd}>&${errfd}" ; shift 1;;
        --showclientoutput ) showclientoutput=1 ; shift 1 ;;
        --showserverrequest ) showserverrequest=1 ; shift 1 ;;
        --serverdisconnect ) serverdisconnect=1; shift 1 ;;
        --serverresponse ) serverresponse=$2 ; shift 2 ;;
        --startfromline ) startfromline=$2 ; shift 2 ;;
	* ) break;
    esac
done
# Remaining arguments are for the client

server_pid=
client_pid=

# On exit clean up 
trap terminate_processes EXIT

# We will exit with the client exit status
status=0

function terminate_processes() {
    if [ "$server_pid" ] ; then
	if [ -d /proc/$server_pid ] ; then 
            echo "$(/usr/bin/date +%S.%N) - Killing off server with pid $server_pid" >&$debugfd
	    kill -9 $server_pid >&/dev/null
	fi
	wait $server_pid >&/dev/null
    fi
    if [ "${showserverrequest}" = 1 ] ; then
        if [ -s /tmp/csse2310.server.$$.out ] ; then
            echo "Server received the following request:"
            /usr/bin/cat /tmp/csse2310.server.$$.out | tail -n +${startfromline}
        else
            echo "Server did not receive a request"
        fi
    fi

    # Remove the named pipes and temporary file
    echo "$(/usr/bin/date +%S.%N) - Removing named pipes and temporary files" >&$debugfd
    /usr/bin/rm -f /tmp/csse2310.client.* /tmp/csse2310.server.* /tmp/csse2310.listen.*
    # Exit with client status 
    return $status
}


# Determine a free port for the server to listen on
port=$(testfiles/freeport.sh)
echo "$(/usr/bin/date +%S.%N) - Identified free port number: $port" >&$debugfd

/usr/bin/rm -f /tmp/2310.server.$$.out
/usr/bin/rm -f /tmp/csse2310.listen.*
# Start dummy server
if [ "${serverdisconnect}" = 1 ] ; then
    /local/courses/csse2310/bin/immediate_disconnection $port > /tmp/csse2310.server.$$.out &
else
    (/usr/bin/cat "${serverresponse}"; sleep 1.5) | /usr/bin/nc -4 -l $port > /tmp/csse2310.server.$$.out \
    2>/dev/null &
fi
server_pid="$!"
echo "$(/usr/bin/date +%S.%N) - Started server - pid $server_pid" >&$debugfd
echo "$(/usr/bin/date +%S.%N) - Waiting for server to listen on port $port ..." >&$debugfd

testfiles/wait_until_listening.sh $server_pid $port 2>&$errfd
echo "$(/usr/bin/date +%S.%N) - Server is listening" >&$debugfd

# Run client - timeout after 3.5 seconds
echo "$(/usr/bin/date +%S.%N) - Starting client" >&$debugfd
(sleep 2; echo -n "") | /home/CSSE2310/timeit -p "${CSSE2310_PRELOAD}" -t 3.5 -k 0.5 \
    -o /dev/null ${client} "$@" ${port} 2>&$errfd > /tmp/csse2310.client.$$.out
# Capture exit status of client
status=$?
echo "$(/usr/bin/date +%S.%N) - Client exited with status $status" >&$debugfd
if [ "${showclientoutput}" = 1 ] ; then
    /usr/bin/cat /tmp/csse2310.client.$$.out | /usr/bin/tail -n +${startfromline}
fi

# This will call terminate_processes - which will kill off the server and tidy up
exit 0
