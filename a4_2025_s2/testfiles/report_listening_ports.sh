#/bin/bash
# report_listening_ports.sh pid
# Echoes on one line the ports that a process is listening on
if [ ! "$1" ] ; then
    exit 1
fi
# Steps
# - file descriptors the process has
# - filter to sockets
# - extract inode
# - look up inode in /proc/self/net/tcp
# - filter to listening sockets only
# - extract the port number being listened on (hex)
# - convert to decimal
# - sort
results=$(ls -l /proc/$1/fd 2>/dev/null | grep socket | cut -d "[" -f 2 | tr -d ] | xargs -I X grep X /proc/self/net/tcp | grep "00000000:0000 0A" | cut -c 16-19 | xargs -I X printf %d\\n 0xX | sort -n)
echo $results
if [ "$results" = "" ] ; then 
    exit 1
else
    exit 0
fi

