#!/bin/bash
declare -i port
port=$RANDOM
port+=1024
while true ; do
    service=$(grep " ${port}/tcp" /etc/services | cut -d ' ' -f 1)
    if [ ! "$service" ] ; then
	port+=1
	continue;
    fi
    rm -f /tmp/nc.out
    nc -v -4 -l ${port} > /tmp/nc.out 2>&1 </dev/null &
    nc_pid=$!
    sleep 0.2
    exec 4</tmp/nc.out
    read line <&4
    read line <&4
    kill $nc_pid >&/dev/null
    wait $nc_pid >&/dev/null
    if echo $line | grep Listening &>/dev/null ; then
	break
    else
	port+=1
    fi
done
rm -f /tmp/nc.out
echo $port $service
