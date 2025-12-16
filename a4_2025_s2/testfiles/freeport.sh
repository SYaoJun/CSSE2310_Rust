#!/usr/bin/bash
declare -i port
port=$RANDOM
port+=1024

while true; do
    (nc -l -4 -p "$port" >/dev/null 2>&1 &) 
    nc_pid=$!
    sleep 0.1

    if ss -tln | grep -q ":$port "; then
        kill $nc_pid &>/dev/null
        wait $nc_pid &>/dev/null
        echo $port
        break
    else
        kill $nc_pid &>/dev/null
        wait $nc_pid &>/dev/null
        port+=1
    fi
done