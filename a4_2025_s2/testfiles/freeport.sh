#!/bin/bash

if command -v python3 >/dev/null 2>&1 ; then
    python3 - <<'PY'
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.bind(('127.0.0.1', 0))
port = s.getsockname()[1]
s.close()
print(port)
PY
    exit 0
fi

declare -i port
port=$RANDOM
port+=1024

while true; do
    (nc -l -4 -p "$port" >/dev/null 2>&1 &)
    nc_pid=$!
    sleep 0.1

    if command -v ss >/dev/null 2>&1 && ss -tln | grep -q ":$port "; then
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