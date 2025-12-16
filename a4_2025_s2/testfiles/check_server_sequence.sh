#!/bin/bash
# check_server_sequence.sh [--valgrind] [--debug] [--allowexit] \
#	    [--stats] [--logmem] maxConnectionsArg greeting \
#               request-sequence-file
#
# Read lines from a file that describes a sequence of operations to be 
# carried out. Lines will have one of the following formats. Clients are 
# numbered from 1 (n prefix below should be replaced by the client number).
#	# - line starting with # is a comment line and is ignored
#	delay n - set delay to given number of seconds (overriding current/
#		default delay (0.1s initially)
#	sighup - SIGHUP signal is sent to the server
#	sigpipe - SIGPIPE signal is sent to the server
#       sigusr1 - SIGUSR1 signal is sent to the server
#       killchild - send TERM signal to any children of server
#	sleep duration - sleep for the given duration
#	n open - connection is opened for client n, but nothing is sent
#		(Note there is no need to open a connection first - sending
#		a message will open a connection if none is open)
#       n sendfile filename - send the given file from the given client
#	n send message - send the given message from the given client. The
#		message is sent with "echo -en" so may contain backslash escape
#		sequences. The connection will be opened if one is not already 
#		open.
#	n close - close connection for client n (i.e. kill off client)
#       n readProtocolResponse [timeout] - read complete protocol response
#               from client connection and summarise it
#       n readHTTPresponse [timeout] - read complete HTTP 
#               response from client connection and summarise it
#	n read [timeout] - read one line from client connection. (If a long delay
#		is expected or possible in client responses then this can be used
#		as a synchronisation mechanism - i.e. don't let the messages to
#		the client get ahead of the responses. read will timeout after
#		the given delay (10s default).
#	n readtimeout [timeout] - attempt to read one line from client 
#		connection - with given timeout (0.5 second default). We expect
#		to get the timeout, i.e. no data available.
#	n openstats - open the STATS port to the server. (This is only 
#		applicable if --stats is specified on the command line. This
#		means we set A4_STATS_PORT environment variable to a free port
#		before running the server.) Connections to the STATS port
#		must be manually created using this command before use
#
# Options to this test program are:
#	--debug - print debugging information to standard error as we go
#       --allowexit - allow the server to exit without aborting test
#	--stats - see description above of openstats
#       --logmem - include testfiles/memory_interposer.so in LD_PRELOAD. This
#               interposer will log memory usage on receipt of SIGUSR1
#
# The standard output of this test program is all of the data received by
# all of the clients (in sequence).
#
# The standard error of this test program is whatever is emitted by the server
# (minus the line containing the port number being listened on)
# (plus debug information if --debug is specified.)
#
# This program also captures data in the following files:
#       /tmp/csse2310.memusage - if testfiles/memory_interposer.so is loaded
#               and SIGUSR1 is sent to the server
#	/tmp/csse2310.activethreadcount.txt - just before reading every command
#		in the sequence file, and at the end of the sequence, we capture
#		how many threads are active in the server (ignoring those that
#		are active at the start). We don't cpature thread counts before
#		comment lines, read* lines or sleep lines - since these lines
#		are waiting for some activity to finish.
#       /tmp/csse2310.activesocketcount.txt - just before reading every command
#               in the sequence file, and at the end of the sequence, we capture
#               how many sockets are open in the server (ignoring those that are
#               open at the start). We don't capture socket counts before
#               comment lines, read* lines or sleep lines - since these lines
#               are waiting for some activity to finish.
#	/tmp/csse2310.totalthreadcount.txt - the total number of threads that were
#		started by the server (ignoring those active at the start)
# The last of these relies on the server being run with the 
# thread_interposer.so LD_PRELOAD (so we can capture appropriate data)
#	
PATH=${PATH}:/usr/bin:/local/courses/csse2310/bin

if test -t  2 ; then
    # stderr is a tty
    normal="$(tput sgr0)"
    bold="$(tput bold)"
    underline="$(tput smul)"
    # Foreground colours
    black="$(tput setaf 0)"
    red="$(tput setaf 1)"
    green="$(tput setaf 2)"
    yellow="$(tput setaf 3)"
    blue="$(tput setaf 4)"
    magenta="$(tput setaf 5)"
    cyan="$(tput setaf 6)"
    white="$(tput setaf 7)"
else 
    normal=""
    bold=""
    underline=""
    black=""
    red=""
    green=""
    yellow=""
    blue=""
    magenta=""
    cyan=""
    white=""
fi

client1="  ${green}${bold}"
client2="  ${red}${bold}"
client3="  ${yellow}${bold}"
client4="  ${magenta}${bold}"
client5="  ${green}${bold}"
client6="  ${red}${bold}"
client7="  ${yellow}${bold}"
client8="  ${magenta}${bold}"
advice="  ${cyan}"
sequence="  ${white}"

baseSocketCount=0

# Usage: set_socket_base_count pid
function set_socket_base_count() {
    baseSocketCount=0
    baseSocketCount=$(count_sockets $1)
}

# Usage: count_sockets pid
function count_sockets() {
    numSockets=$(ls -l /proc/$1/fd 2>/dev/null | grep socket | sed -e 's/^.*socket://' \
        | sort | uniq | wc -l)
    echo $((numSockets - baseSocketCount))
}

# Usage: count_active_threads pid basecount
function count_active_threads() {
    numthreads=$(ls /proc/$1/task 2>/dev/null | wc -l)
    basecount=$2
    echo $((numthreads - basecount))
}

# Requires use of thread_interposer
# Usage: count_total_threads basecount
function count_total_threads() {
    numthreads=$(cat /tmp/csse2310.threadcount)
    basecount=$1
    echo $((numthreads - basecount))
}

nc_pids=()
clients=()

trap cleanup EXIT

server_status=0
function cleanup() {
    terminate_processes
    rm -f /tmp/csse2310.client.$$.* /tmp/csse2310.server.out.$$ /tmp/csse2310.server.err.$$ 
    exit $server_status
}

function is_client_alive() {
    [ -d /proc/${nc_pids[$1]} ]
}

function debug() {
    echo "${!1}${2}${normal}" >&${debugfd}
}

function terminate_processes() {
    # Copy any remaining text from output pipes to the client's stdout file
    catpids=()
    for i in ${!nc_pids[@]} ; do
        if [ -r /tmp/csse2310.client.$$.$i.pipe.out ] ; then
            debug client$i "Reading remaining data on client $i stdout"
            timeit -stdin /tmp/csse2310.client.$$.$i.pipe.out -t 2 -k 1 -o /dev/null cat >> /tmp/csse2310.client.$$.$i.stdout &
            catpids[$i]=$!
        fi
    done
    sleep 0.02

    # Kill off the clients (if any) and server
    if [ ${#nc_pids[@]} -gt 0 ] ; then
        pids="${nc_pids[@]}"
	debug advice "Killing off ${#nc_pids[@]} clients (pids ${pids})"
	kill -9 ${nc_pids[@]} >&/dev/null
	wait ${nc_pids[@]} >&/dev/null
	unset nc_pids
    fi
    if [ "$server_pid" ] ; then
	if ps -p $server_pid > /dev/null ; then
	    debug advice "Killing off server (pid $server_pid)"
	    kill -TERM $server_pid >&/dev/null || kill -KILL $server_pid >&/dev/null
	fi
	wait $server_pid >&/dev/null
	status=$?
        debug server "Server exited with status $status"
	case "$status" in
	    137) ;; # TERM - ignore
	    139) echo "Server died due to Segmentation Fault" >&2 ;;
	    141) echo "Server died due to SIGPIPE" >&2 ;;
	    143) ;; # KILL - ignore
	    134) echo "Server died due to Abort (possible memory error?)" >&2;;
            *) server_status=$status; ;;
	esac
	unset server_pid
    fi

    # Wait for any remaining cat processes to finish (they should die without
    # needing to be killed)
    if [ ${#catpids[@]} -gt 0 ] ; then
        pids="${catpids[@]}"
        debug advice "Waiting for all client output to be saved (pids ${pids})"
	wait ${catpids[@]} >&/dev/null
    fi

    # Remove the named pipes
    debug advice "Cleaning up"
    rm -f /tmp/csse2310.client.$$.*.pipe.*
}

exec {debugfd}>/dev/null
connlimit=0
delay=0.03
allowexit=""
logmem=""
valgrind=""
unset A4_STATS_PORT statsport
while true ; do
    case "$1" in 
        --valgrind ) valgrind=1 ; shift 1 ;;
	--debug ) eval "exec ${debugfd}>&2" ; shift 1 ;;
        --allowexit ) allowexit=1 ; shift 1 ;;
        --logmem ) logmem="testfiles/memory_interposer.so:"; shift ;;
	--stats )
	    statsport=$(testfiles/freeport.sh)
	    export A4_STATS_PORT=${statsport}
	    shift;;
	* ) break;
    esac
done
if [ "$1" ] ; then
    maxConnsArg="$1"
    shift
else
    echo "Maximum connections argument not provided" >&2
    exit 1
fi
if [ "$1" ] ; then
    greeting="$1"
    shift
else
    echo "Greeting argument not provided" >&2
    exit 1
fi

# Check sequence file exists
if [ ! -r "$1" ] ; then
    echo "No operation file provided" >&2
    exit 1
else
    # Read from fd 3 to retrieve operations
    exec 3< "$1"
fi

# Determine a free port for the server to listen on
port=$(testfiles/freeport.sh)
debug advice "Identified free port number for server to use: $port"

# Start up the server being tested in the background and wait for it to
# be listening. We remove temporary files created by interposer
rm -f /tmp/csse2310.listen.*
rm -f /tmp/csse2310.memusage
debug advice "Starting server in the background as follows:"

if [ "${valgrind}" ] ; then
    debug advice "/usr/bin/valgrind ./ratsserver "${maxConnsArg}" "${greeting}" $port &"
    /usr/bin/valgrind ${ratsserver:=./ratsserver} "${maxConnsArg}" "${greeting}" $port >/tmp/csse2310.server.out.$$ &
    server_pid=$!
else 
    debug advice "LD_PRELOAD=${logmem}testfiles/thread_interposer.so:${CSSE2310_PRELOAD} ./ratsserver "${maxConnsArg}" "${greeting}" $port &"
    LD_PRELOAD=${logmem}testfiles/thread_interposer.so:${CSSE2310_PRELOAD} ${ratsserver:=./ratsserver} "${maxConnsArg}" "${greeting}" $port >/tmp/csse2310.server.out.$$ 2>/tmp/csse2310.server.err.$$ &
    server_pid=$!
fi

if ! testfiles/wait_until_listening.sh $server_pid $port ; then
    debug advice "Server not listening as expected - aborting test"
    exit 1
fi

# Wait for port number on standard error
found=""
for ((i=0; i < 25; i++)) ; do
    if grep $port /tmp/csse2310.server.err.$$ >&/dev/null ; then
        debug advice "Got port number on standard error"
        found=1
        break;
    fi
    sleep 0.02
done
if [ ! "$found" ] ; then
    debug advice "Did not see port number on standard error - attempting to continue"
fi

baseactivethreadcount=$(count_active_threads $server_pid 0)
basetotalthreadcount=$(count_total_threads 0)
set_socket_base_count $server_pid
debug advice "Started server on port $port."
debug advice "Active thread count (base) = $baseactivethreadcount"
debug advice "Total thread count (base) = $basetotalthreadcount"
debug advice "Total socket count (base) = $baseSocketCount"

# Headers for our thread count files (ensures previous data wiped also).
rm -f /tmp/csse2310.totalthreadcount.txt /tmp/csse2310.activethreadcount.txt \
    /tmp/csse2310.activesocketcount.txt
echo "Thread counts reported before each request line. (0 assumed before any clients created.)" > /tmp/csse2310.activethreadcount.txt
echo "Total threads created since listen" > /tmp/csse2310.totalthreadcount.txt
echo "Socket count reported before each request line. (0 assumed before any clients created.)" > /tmp/csse2310.activesocketcount.txt

start=$(date +%s.%N)
declare -i linenum=0;
# Read each line in the operations file
while read -r client request mesg <&3 ; do
    linenum+=1
    debug sequence "Time $(bc <<< "$(date +%s.%N) - ${start}") - Read line from sequence file: $client $request $mesg"
    if [[ $client =~ ^#.*$ ]] ; then
	# Skip over comments
	debug advice "Skipping comment"
	continue;
    fi

    # Make sure server is still alive
    if [ ! "$allowexit" -a ! -d /proc/$server_pid ] ; then
	echo "Server has died unexpectedly - aborting" >&2
	terminate_processes
	exit 10
    fi
    # Output active thread count and socket count if this isn't a read request or
    # a sleep request
    if ! [[ "$request" =~ read ]] && ! [[ "$client" =~ sleep ]] ; then 
	echo Line $linenum: $(count_active_threads $server_pid $baseactivethreadcount) >> /tmp/csse2310.activethreadcount.txt
        echo Line $linenum: $(count_sockets $server_pid) >> /tmp/csse2310.activesocketcount.txt
    fi

    case $client in 
	sleep )
	    debug advice "Sleeping for ${request} seconds"
	    sleep ${request}
	    # Skip everything else (e.g. we don't output a thread count)
	    continue;
	    ;;
	delay )
	    delay=${request}
	    continue;
	    ;;
	sighup )
	    debug advice "Sending SIGHUP to server"
	    kill -HUP $server_pid
	    sleep 0.03
	    continue
	    ;;
	sigpipe )
	    debug advice "Sending SIGPIPE to server"
	    kill -PIPE $server_pid
	    sleep 0.03
	    continue
	    ;;
	sigusr1 )
	    debug advice "Sending SIGUSR1 to server"
	    kill -USR1 $server_pid
	    sleep 0.03
	    continue
	    ;;
        killchild )
            debug advice "Sending SIGTERM to children of server"
            pkill -TERM -P $server_pid
            sleep 0.03
            continue
            ;;
	* )
	    ;;
    esac

    # Work out the named pipe files for communicating with this client
    pipein=/tmp/csse2310.client.$$.${client}.pipe.in
    pipeout=/tmp/csse2310.client.$$.${client}.pipe.out
    if [ ! -p ${pipein} ] ; then
	# Input pipe doesn't exist (we assume the same is true of output pipe)
	# This means the client does not exist
	if [ "${request}" = "close" ] ; then
	    # The first time we've seen this client is with a close request
	    # - ignore this line
	    debug advice "Ignoring close request for client we haven't seen before"
	    continue
	fi
	if [ "${request}" = "openstats" ] ; then
	    if [ ! "${statsport}" ] ; then
		echo "STATS port not enabled but request made to open STATS port" >&2
		exit 1
	    fi
	    port_to_use=${statsport}
	else
	    port_to_use=${port}
	fi
	# Create named pipes for new client comms
	mkfifo ${pipein}
	mkfifo ${pipeout}
	# Make sure we keep the pipes open for writing. (We open for reading
	# and writing because opening in one direction blocks on named pipes.)
	exec 44<>${pipein}
	exec 45<>${pipeout}
	# Start up netcat as our dummy client - anything received over the 
	# input pipe will be sent to the server. 
        debug client${client} "Starting nc (2310netclient) as client ${client}"
	2310netclient ${port_to_use} < ${pipein} > ${pipeout} 2>/dev/null &
	nc_pids[${client}]="$!"
	# Create an empty client output file
	rm -f /tmp/csse2310.client.$$.${client}.stdout
	touch /tmp/csse2310.client.$$.${client}.stdout
	clients+=("${client}")
	# netcat will have inherited fds 44 and 45 so we can close them here
	exec 44>&- 45>&-
    fi
    case "${request}" in
	close )
	    # Copy everything remaining from output pipe to stdout file for client
	    debug client${client} "Saving everything from client's stdout and quitting client"
	    timeit -t 2 -k 1 -o /dev/null cat ${pipeout} >> /tmp/csse2310.client.$$.${client}.stdout &
	    catpid=$!
	    # Need a delay here to (hopefully) start cat reading before we kill the
	    # client (which kills the other end of the pipe)
	    sleep 0.03
	    # Kill off the client
	    kill -9 ${nc_pids[${client}]} >&/dev/null
	    wait ${nc_pids[${client}]} >&/dev/null
	    wait $catpid >&/dev/null
	    rm -f ${pipein} ${pipeout}
	    unset nc_pids[${client}]
	    continue
	    ;;
        readHTTPresponse )
            if ! is_client_alive ${client} ; then
                debug client${client} "Client is dead - can't read HTTP response"
                echo "Client has exited - can't read HTTP response" >> /tmp/csse2310.client.$$.${client}.stdout
                continue
            fi
            timeout=0.4
            if [ "$mesg" ] ; then
                timeout="$mesg"
	    fi
            debug client${client} "Attempting to read complete HTTP response from client ${client}'s stdout (timeout ${timeout})"
            echo "HTTP response:" >> /tmp/csse2310.client.$$.${client}.stdout
            if ! timeit -t ${timeout} -k 1 -o /dev/null summarise_http_response < ${pipeout} >> /tmp/csse2310.client.$$.${client}.stdout ; then
                echo " - Failed to get response" >> /tmp/csse2310.client.$$.${client}.stdout
                debug client${client} "Failed to get response"
            else
                debug client${client} "Got response"
            fi
            continue
            ;;
        readProtocolResponse )
            if ! is_client_alive ${client} ; then
                debug client${client} "Client is dead - can't read protocol response"
                echo "Client has exited - can't read protocol response" >> /tmp/csse2310.client.$$.${client}.stdout
                continue
            fi
            timeout=0.4
            if [ "$mesg" ] ; then
                timeout="$mesg"
	    fi
            debug client${client} "Attempting to read complete protocol response from client ${client}'s stdout (timeout ${timeout})"
            echo "Protocol response:" >> /tmp/csse2310.client.$$.${client}.stdout
            if ! timeit -t ${timeout} -k 1 -o /dev/null testfiles/dump_message.sh < ${pipeout} >> /tmp/csse2310.client.$$.${client}.stdout ; then
                echo " - Failed to get response" >> /tmp/csse2310.client.$$.${client}.stdout
                debug client${client} "Failed to get response"
            else
                debug client${client} "Got response"
            fi
            continue
            ;;
	read )
            if ! is_client_alive ${client} ; then
                debug client${client} "Client is dead - can't read line"
                echo "Client has exited - can't read line" >> /tmp/csse2310.client.$$.${client}.stdout
                continue
            fi
	    unset clientline
	    if [ "$mesg" ] ; then
		timeout="${mesg}"
	    else
		timeout=0.2
	    fi
	    debug client${client} "Attempting to read line from client ${client}'s stdout"
	    IFS=""
	    if read -t ${timeout} -r clientline < ${pipeout} ; then
		# Save the line to the client's output file
		debug client${client} "Client ${client} output line '${clientline}'"
		echo "${clientline}" >> /tmp/csse2310.client.$$.${client}.stdout
	    else
		# Got timeout which was not expected
		echo "Got unexpected timeout (${timeout}s) waiting for line from server" >> /tmp/csse2310.client.$$.${client}.stdout
		debug client${client} "Unexpected timeout (${timeout}s) waiting for client ${client} to output line"
	    fi
	    unset IFS
	    continue
	    ;;
	readtimeout )
	    unset clientline
	    # We open pipe for reading and writing here so there is no block
	    if [ "$mesg" ] ; then
		timeout="${mesg}"
	    else
		timeout=0.1
	    fi
	    IFS=""
	    if read -t "${timeout}" -r clientline <> ${pipeout} ; then
		# we expected no data but got some
		debug client${client} "Expected client ${client} to output nothing in the next ${timeout}s but got line '${clientline}'"
		# Save the line to the client's output file
		echo "Expected ${timeout}s timeout on read, but got unexpected: ${clientline}" >> /tmp/csse2310.client.$$.${client}.stdout
	    else
		echo "Waited ${timeout}s - nothing arrived from server (as expected)" >> /tmp/csse2310.client.$$.${client}.stdout
		debug client${client} "Client ${client} output nothing in the next ${timeout}s (as expected)"
	    fi
	    unset IFS
	    continue
	    ;;
        sendfile )
            if is_client_alive ${client} ; then 
                debug client${client} "Sending file ${mesg} to client ${client}'s stdin"
                cat "${mesg}" > ${pipein}
            else
                debug client${client} "Can't send file ${mesg} to client ${client}'s stdin - client is dead"
            fi
            ;;
	send )
            if is_client_alive ${client} ; then 
                debug client${client} "Sending '${mesg}' to client ${client}'s stdin"
                echo -en "${mesg}" > ${pipein}
            else
                debug client${client} "Can't send '${mesg}' to client ${client}'s stdin - client is dead"
            fi
	    ;;
        open | openstats )
            # Have been dealt with
            ;;
	*) # Unknown
            echo "Unknown request ($request)" >&2
            exit 23
	    ;;
    esac
    sleep "${delay}"
done

# Allow the clients to finish (necessary if $delay is small
#echo "Waiting until clients finish" >&${debugfd}
#sleep 0.5

threadcount=$(count_active_threads $server_pid $baseactivethreadcount)
echo End: $threadcount >> /tmp/csse2310.activethreadcount.txt
debug advice "Additional threads: $threadcount" >&${debugfd}
socketcount=$(count_sockets $server_pid)
echo "End: $socketcount" >> /tmp/csse2310.activesocketcount.txt
debug advice "Additional sockets: $socketcount" >&${debugfd}
threadcount=$(count_total_threads $basetotalthreadcount)
echo $threadcount >> /tmp/csse2310.totalthreadcount.txt
debug advice "Total threads since listen: $threadcount" >&${debugfd}

# Have now completed the operations - kill off processes and remove pipes
terminate_processes

if [ -s /tmp/csse2310.server.out.$$ ] ; then
    echo "Output from server is:"
    echo "------------------------"
    cat /tmp/csse2310.server.out.$$
    echo "------------------------"
fi

# Send the output from the clients to stdout
for client in ${clients[@]} ; do
    echo "Output from client $client is:"
    echo "------------------------"
    cat /tmp/csse2310.client.$$.${client}.stdout
    echo "------------------------"
done

# Send server's stderr to stderr (excluding the line with the 
# port number on it)
grep -v ${port} /tmp/csse2310.server.err.$$ >&2

# Remove any temporary files
rm -f /tmp/csse2310.client.$$.* /tmp/csse2310.server.out.$$ /tmp/csse2310.server.err.$$ /tmp/csse2310.listen.*
exit 0
