#!/bin/bash
# Check that server reports being unable to listen on an already occupied 
# port

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

if [ "$1" = "-service" ] ; then
    service=($(testfiles/freeservice.sh))
    # service name to use
    portarg=${service[1]}
    port=${service[0]}
else 
    portarg=$(testfiles/freeport.sh)
    port=${portarg}
fi
sleep 0.2

# Start a dummy server listening on this port in the background.
# Prefer python3 for portability/reliability across platforms.
if command -v python3 >/dev/null 2>&1 ; then
    python3 - <<PY &
import socket, time
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(('127.0.0.1', int('${port}')))
s.listen(1)
time.sleep(10)
PY
    nc_pid=$!
else
    # GNU netcat supports --no-shutdown; BSD netcat (macOS) does not.
    # -k keeps listening for multiple connects on BSD netcat.
    nc -4 -l -k ${port} >&/dev/null </dev/null &
    nc_pid=$!
fi

# Wait for the dummy server to be listening on this port
testfiles/wait_until_listening.sh $nc_pid $port

# Start up server and try to listen on this port
run_with_timeout 3 ${ratsserver:=./ratsserver} 0 hi ${portarg} 2>/tmp/2310.server.$$.out
status=$?
# Remove the port number from any error message
sed -e "s/${portarg}/PORTNUM/" < /tmp/2310.server.$$.out >&2
rm -f /tmp/2310.server.$$.out

# Kill off dummy server
kill -9 $nc_pid >&/dev/null
wait $nc_pid >&/dev/null

exit $status
