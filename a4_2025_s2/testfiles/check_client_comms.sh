#!/bin/bash
# check_client_comms.sh [--debug] [--showclientoutput] \
#       [--serverdisconnect | --showserverrequest ] \
#       [--startfromline n] \
#       [--serverresponse response-file] [client-arguments ...]
# (Client port number argument will be supplied by this script)
PATH=/usr/bin:/bin:${PATH}

# macOS ships bash 3.2 by default which does not support bash4+ dynamic fd
# syntax like: exec {errfd}>&2
errfd=2
debugfd=/dev/null

client=${ratsclient:=./ratsclient}
if [ ! -x "$client" ] && [ -x "../target/debug/ratsclient" ] ; then
    client="../target/debug/ratsclient"
fi
showclientoutput=0
showserverrequest=0
serverexpectshttp=0
serverdisconnect=0
serverresponse=/dev/null
# "startfromline" applies to the client output and the server request output
startfromline=1

run_with_timeout() {
    local seconds="$1"
    shift

    if command -v timeout &> /dev/null; then
        timeout "$seconds" "$@"
        return $?
    fi

    if command -v gtimeout &> /dev/null; then
        gtimeout "$seconds" "$@"
        return $?
    fi

    perl -e '
        my $t = shift @ARGV;
        my $pid = fork();
        if (!defined $pid) { exit 125; }
        if ($pid == 0) {
            exec @ARGV;
            exit 127;
        }
        $SIG{ALRM} = sub { kill 9, $pid; exit 124; };
        alarm($t);
        waitpid($pid, 0);
        my $status = $?;
        alarm(0);
        exit($status >> 8);
    ' "$seconds" "$@"
    return $?
}

# Process command line arguments
while true ; do
    case "$1" in 
	--debug ) debugfd=/dev/stderr ; shift 1;;
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
	if ps -p "$server_pid" >/dev/null 2>&1 ; then 
	    echo "$(date +%S.%N) - Killing off server with pid $server_pid" >"$debugfd"
	    kill -9 $server_pid >&/dev/null
	fi
	wait $server_pid >&/dev/null 2>&1
    fi
    if [ "${showserverrequest}" = 1 ] ; then
        if [ -s /tmp/csse2310.server.$$.out ] ; then
            echo "Server received the following request:"
            cat /tmp/csse2310.server.$$.out | tail -n +${startfromline}
        else
            echo "Server did not receive a request"
        fi
    fi

    # Remove the named pipes and temporary file
    echo "$(date +%S.%N) - Removing named pipes and temporary files" >"$debugfd"
    rm -f /tmp/csse2310.client.* /tmp/csse2310.server.* /tmp/csse2310.listen.*
    # Exit with client status 
    return $status
}


# Determine a free port for the server to listen on
port=$(testfiles/freeport.sh)
echo "$(date +%S.%N) - Identified free port number: $port" >"$debugfd"

rm -f /tmp/2310.server.$$.out
rm -f /tmp/csse2310.listen.*
# Start dummy server
if [ "${serverdisconnect}" = 1 ] ; then
    /local/courses/csse2310/bin/immediate_disconnection $port > /tmp/csse2310.server.$$.out &
else
    (cat "${serverresponse}"; sleep 1.5) | /usr/bin/nc -4 -l $port > /tmp/csse2310.server.$$.out \
    2>/dev/null &
fi
server_pid="$!"
echo "$(date +%S.%N) - Started server - pid $server_pid" >"$debugfd"
echo "$(date +%S.%N) - Waiting for server to listen on port $port ..." >"$debugfd"

testfiles/wait_until_listening.sh $server_pid $port 2>&$errfd
echo "$(date +%S.%N) - Server is listening" >"$debugfd"

# Run client - timeout after 3.5 seconds
echo "$(date +%S.%N) - Starting client" >"$debugfd"
run_with_timeout 4 ${client} "$@" ${port} 2>&$errfd > /tmp/csse2310.client.$$.out
# Capture exit status of client
status=$?
echo "$(date +%S.%N) - Client exited with status $status" >"$debugfd"
if [ "${showclientoutput}" = 1 ] ; then
    cat /tmp/csse2310.client.$$.out | tail -n +${startfromline}
fi

# This will call terminate_processes - which will kill off the server and tidy up
exit 0
