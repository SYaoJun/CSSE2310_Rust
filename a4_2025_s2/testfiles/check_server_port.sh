#!/bin/bash
# Check that server reports the correct port number to stderr
# If an argument is given, it must be a port number to listen on (may be 0)
# If no port number is given, we choose one randomly.
# If the ame "NUM" is given, we also choose one randomly
# If the name "NONE" is given then we don't supply this arg to the server
# If the name "NAME" is given then we use a service name instead
 
rm -f /tmp/stderr

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

if [ ! -x "./ratsserver" ] && [ -x "../target/debug/ratsserver" ] ; then
    ratsserver="../target/debug/ratsserver"
fi

if [ "$1" = "NONE" ] ; then
    portarg=""
    port=0
elif [ "$1" = "NAME" ] ; then
    service=($(testfiles/freeservice.sh))
    # service name to use
    portarg=${service[1]}
    port=${service[0]}
elif [ "$1" = "NUM" ] ; then
    # Choose a random free port
    portarg=$(testfiles/freeport.sh)
    port=$portarg
    sleep 0.2
elif [ "$1" ] ; then
    # Port number given on the command line
    portarg=$1
    port=$1
else
    # Choose a random free port
    portarg=$(testfiles/freeport.sh)
    port=$portarg
    sleep 0.2
fi

# Start up server in the background and wait for it to be listening
if [ "${portarg}" ] ; then
    ${ratsserver:=./ratsserver} 0 hi ${portarg} 2>/tmp/stderr &
    server_pid=$!
else
    # No port given
    ${ratsserver:=./ratsserver} 0 hi 2>/tmp/stderr &
    server_pid=$!
fi
testfiles/wait_until_listening.sh "$server_pid" "$port"

# Open the server's stderr for reading (on fd 4)
exec 4</tmp/stderr
# Read the first line from that stderr (should contain a port number)
read reported_port <&4

if [ "$port" != 0 ] ; then
    # If we asked for a specific port, make sure that number was output
    if ! echo "$reported_port" | grep "$port" >&/dev/null || [ "$port" -ne "${reported_port}" ] 2>/dev/null; then
	echo "Reported port incorrect - expected $port got $reported_port" >&2
	status=1
    else
	echo "Reported port OK" >&2
	status=0
    fi
else 
    # We asked for port zero - just make sure we got a number - we delete
    # all the digits in the response and make sure nothing is left
    p=$(echo $reported_port | tr -d 0-9)
    if [ ! "$p" ] ; then
	echo "Reported port is numeric" >&2
	status=0
    else
	echo "Reported port non numeric - got $reported_port" >&2
	status=2
    fi
fi

if [ "$status" = 0 ] ; then 
    # If the reported port number was OK, see if netcat can connect
    # We start netcat in verbose mode so it reports a connection message if
    # it connects.
    rm -f /tmp/stderr2
    run_with_timeout 2 nc -4 -v localhost $reported_port >/dev/null 2>/tmp/stderr2

    # GNU netcat (Linux) typically prints: "Connected to 127.0.0.1"
    # BSD netcat (macOS) typically prints: "Connection to ... succeeded!"
    if grep "Connected to 127.0.0.1" /tmp/stderr2 >&/dev/null || \
       grep "succeeded" /tmp/stderr2 >&/dev/null || \
       grep "open" /tmp/stderr2 >&/dev/null ; then
	echo "Test client connected to server"
    else
	echo "Test client failed to connect to server"
	status=3
    fi
fi

# Kill off the server under test
kill -TERM $server_pid >&/dev/null || kill -KILL $server_pid >&/dev/null
wait $server_pid >&/dev/null

# Output server's standard error to standard error here (minus the first line)
tail -n +2 /tmp/stderr >&2
rm -f /tmp/stderr*

exit "$status"
