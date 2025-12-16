#!/usr/bin/bash
# check_client_can_connect.sh [-service]
# Checks whether ratsclient can connect to a server. We use netcat (nc) as a
# dummy server. If "-service" is given on the command line then we 
# we attempt to connect using a service name rather than port number.

rm -f /tmp/$$.out

# Get a free port to listen on
if [ "$1" = "-service" ] ; then
    service=($(testfiles/freeservice.sh))
    name=${service[1]}
    port=${service[0]}
else 
    port=$(testfiles/freeport.sh)
    name=$port
    # echo "port: $port"
fi
# Run netcat as a dummy server listening on that port. We use verbose mode
# so it will report any connections
nc -v -l -4 ${port} < /dev/null 1>/dev/null 2>/tmp/$$.out  &
netcat_pid=$!
# Make sure nc is listening
if ! testfiles/wait_until_listening.sh ${netcat_pid} ${port} ; then
    echo "Dummy server failed to listen - aborting" >&2
fi
# Run the client in the background
LD_PRELOAD="${CSSE2310_PRELOAD}" ${ratsclient:=./ratsclient} "player" "game" "$name" >& /dev/null &
client_pid=$!
sleep 0.4
testfiles/wait_for_connection.sh $client_pid $port

sleep 0.1

# Kill the client
kill $client_pid &>/dev/null
wait $client_pid &>/dev/null
# Kill the server
kill $netcat_pid &>/dev/null
wait $netcat_pid &>/dev/null
sync
sleep 0.5

# Check whether the server reported a connection from the client or not 
if grep "Connection" /tmp/$$.out >&/dev/null ; then
    echo Got connection >&2
    result=0
else
    echo Server did not report connection >&2
    result=1
fi
rm -f /tmp/$$.out
exit $result
