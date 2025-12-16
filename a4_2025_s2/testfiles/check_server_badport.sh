#!/bin/bash
# Check that server reports being unable to listen on an already occupied 
# port

rm -f /tmp/stderr

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

# Start a dummy server listening on this port in the background
nc --no-shutdown -4 -l ${port} >&/dev/null </dev/null &
nc_pid=$!
# Wait for the dummy server to be listening on this port
testfiles/wait_until_listening.sh $nc_pid $port

# Start up server and try to listen on this port
timeout 3 ${ratsserver:=./ratsserver} 0 hi ${portarg} 2>/tmp/2310.server.$$.out
status=$?
# Remove the port number from any error message
sed -e "s/${portarg}/PORTNUM/" < /tmp/2310.server.$$.out >&2
rm -f /tmp/2310.server.$$.out

# Kill off dummy server
kill -9 $nc_pid >&/dev/null
wait $nc_pid >&/dev/null

exit $status
