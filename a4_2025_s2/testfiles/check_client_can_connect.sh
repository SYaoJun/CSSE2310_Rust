#!/bin/bash
# check_client_can_connect.sh [-service]
# Checks whether ratsclient can connect to a server. We use netcat (nc) as a
# dummy server. If "-service" is given on the command line then we 
# we attempt to connect using a service name rather than port number.

rm -f /tmp/$$.out

if [ ! -x "./ratsclient" ] && [ -x "../target/debug/ratsclient" ] ; then
    ratsclient="../target/debug/ratsclient"
fi

is_darwin=0
if [ "$(uname -s 2>/dev/null)" = "Darwin" ] ; then
    is_darwin=1
fi

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
# Prefer a python3 dummy server for portability.
connected_file="/tmp/csse2310.clientconnect.$$.ok"
rm -f "$connected_file"

if command -v python3 >/dev/null 2>&1 ; then
    python3 - "$port" "$connected_file" <<'PY' &
import socket, sys
port = int(sys.argv[1])
flag = sys.argv[2]
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(('127.0.0.1', port))
s.listen(1)
s.settimeout(3.0)
try:
    conn, addr = s.accept()
    with open(flag, 'w') as f:
        f.write('ok\n')
    conn.close()
except Exception:
    pass
finally:
    try:
        s.close()
    except Exception:
        pass
PY
    netcat_pid=$!
else
    # Fallback to netcat.
    nc -v -k -l -4 ${port} < /dev/null 1>/dev/null 2>/tmp/$$.out  &
    netcat_pid=$!
fi

# Make sure dummy server is listening
if ! testfiles/wait_until_listening.sh ${netcat_pid} ${port} ; then
    echo "Dummy server failed to listen - aborting" >&2
fi
# Run the client in the background
if [ $is_darwin -eq 1 ] ; then
    ${ratsclient:=./ratsclient} "player" "game" "$name" >& /dev/null &
else
    LD_PRELOAD="${CSSE2310_PRELOAD}" ${ratsclient:=./ratsclient} "player" "game" "$name" >& /dev/null &
fi
client_pid=$!
sleep 0.4

# Wait briefly for accept() marker file.
conn_status=1
for i in $(seq 1 30); do
    if [ -f "$connected_file" ]; then
        conn_status=0
        break
    fi
    sleep 0.1
done

sleep 0.1

# Kill the client
kill $client_pid &>/dev/null
wait $client_pid &>/dev/null
# Kill the server
kill $netcat_pid &>/dev/null
wait $netcat_pid &>/dev/null
sync
sleep 0.5

# Prefer the direct connection check result (portable). Some nc variants do not
# reliably report connection messages in verbose mode.
if [ "$conn_status" -eq 0 ]; then
    echo Got connection >&2
    result=0
else
    # Fallback to checking netcat output
    if grep -i "connect" /tmp/$$.out >&/dev/null || grep -i "succeeded" /tmp/$$.out >&/dev/null || grep -i "open" /tmp/$$.out >&/dev/null ; then
        echo Got connection >&2
        result=0
    else
        echo Server did not report connection >&2
        result=1
    fi
fi
rm -f /tmp/$$.out
rm -f "$connected_file"
exit $result
