#!/bin/bash
# check_client_sequence.sh [--debug] request-sequence-file
#
# Read lines from a file that describes a sequence of operations to be 
# carried out. Lines will have one of the following formats. 
#	# - line starting with # is a comment line and is ignored
#	delay n - set delay to given number of seconds (overriding current/
#		default delay (0.1s initially)
#	sigpipe - SIGPIPE signal is sent to the client
#	sleep duration - sleep for the given duration
#       client send message - send the given message to the client's stdin
#               (sent with `echo -en` so may contain backslash sequences
#       server send message - send the given message from the server back to the
#               client (sent with `echo -en` so can include backslash sequences
#       client read [timeout] - read one line sent from the client stdout
#       server read [timeout] - read one line sent from the client to the server
#	client readtimeout [timeout] - attempt to read one line from client 
#		stdout - with given timeout (0.5 second default). We expect
#		to get the timeout, i.e. no data available.
#	server readtimeout [timeout] - attempt to read one line sent from client to
#		server - with given timeout (0.5 second default). We expect
#		to get the timeout, i.e. no data available.
#       client close - close stdin to the client
#       server close - kill off the server
#       client expectexit - expect the client to have exited
#
clientProg=${ratsclient:=./ratsclient}

if test -t 2 ; then
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

# Make a copy of stderr and close it (to prevent job control messages)
exec {errfd}>&2
exec 2>&-

client="  ${green}${bold}"
server="  ${red}${bold}"
advice="  ${cyan}"
sequence="  ${white}"

server_pid=""
client_pid=""
client_started=""
server_started=""
client_reaped=""
server_reaped=""

trap cleanup EXIT

client_exit_status=99

function cleanup() {
    debug advice "Cleaning up"
    terminate_processes
    # Show the client's stderr
    cat /tmp/csse2310.client.stderr.$$ >&${errfd}
    rm -f /tmp/csse2310.{client,server}.{in,out,stdout,stderr}.$$
    case "$client_exit_status" in
        130) client_exit_status=0;; # SIGINT - ignore
        137) client_exit_status=0;; # KILL - ignore
        139) echo "Client died due to Segmentation Fault" >&${errfd} ;;
        141) echo "Client died due to SIGPIPE" >&${errfd} ;;
        143) client_exit_status=0;; # TERM - ignore
        134) echo "Client died due to Abort (possible memory error?)" >&${errfd};;
    esac
    exit $client_exit_status
}

function is_alive() {
    [ -d /proc/$1 ]
}

function debug() {
    echo "${!1}${2}${normal}" >&${debugfd}
}

function get_remaining_data_from() {
    who="$1"
    pid="$2"
    infdname=${who}infd
    outfdname=${who}outfd
    infd=${!infdname}
    outfd=${!outfdname}
    startedname=${who}_started
    started=${!startedname}
    reapedname=${who}_reaped
    reaped=${!reapedname}

    # Close the input
    eval "exec ${infd}>&-"
    if [ "$started" -a ! "$reaped" ] ; then 
        # Kill the process (may already be dead)
        debug ${who} "Killing ${who} (${pid}) if not already dead"
        if is_alive ${pid} ; then
            debug ${who} "${who} is alive"
            kill -9 ${pid} >&/dev/null
        else
            debug ${who} "${who} is dead"
        fi
        # Reap it if started
        wait ${pid} >&/dev/null
        status=$?
        if [ "$who" = "client" ] ; then
            client_exit_status=$status
        fi
        eval ${reapedname}=1
        debug ${who} "Got exit status $status"
        # Get the output
        debug ${who} "Saving everything from ${who}'s stdout"
        timeout 1 cat <&${outfd} >> /tmp/csse2310.${who}.stdout.$$
        # Close the output
        eval "exec ${outfd}>&-"
        return $status
    fi
    return 0 # Not started or already reaped
}

function terminate_processes() {
    get_remaining_data_from client ${client_pid}
    get_remaining_data_from server ${server_pid}
}

function unknown_request() {
    echo "Unknown request in sequence file: '$@'" >&${errfd}
    exit 23
}

exec {debugfd}>/dev/null
maxArgs=""
connlimit=0
delay=0.01

while true ; do
    case "$1" in 
	--debug ) eval "exec ${debugfd}>&${errfd}" ; shift 1 ;;
	* ) break;
    esac
done

# Check sequence file exists
if [ ! -r "$1" ] ; then
    echo "No operation file provided" >&${errfd}
    exit 23
else
    # Read from fd 3 to retrieve operations
    exec 3< "$1"
fi
shift

# Remaining command line arguments are for the client

# Determine a free port for the server to listen on
port=$(testfiles/freeport.sh)
debug advice "Identified free port number for server to use: $port"

# Remove our output files and pipes if they exist
/usr/bin/rm -f /tmp/csse2310.{client,server}.{stdout,stderr,in,out}.$$ 

# Create the named pipe files for communicating with the server and client
serverin=/tmp/csse2310.server.in.$$
serverout=/tmp/csse2310.server.out.$$
clientin=/tmp/csse2310.client.in.$$
clientout=/tmp/csse2310.client.out.$$
mkfifo ${serverin}
mkfifo ${serverout}
mkfifo ${clientin}
mkfifo ${clientout}
# stderr files
servererr=/tmp/csse2310.server.stderr.$$
clienterr=/tmp/csse2310.client.stderr.$$

# Start up the dummy server in the background and wait for it to be listening. 
debug advice "Starting server in the background as follows:"
debug advice "/usr/bin/nc -k -4 -l ${port}"
/usr/bin/nc -k -4 -l ${port} < ${serverin} > ${serverout} 2>${errfd} &
server_pid=$!
server_processid=${server_pid}
server_started=1

# Need to open the pipes before nc will listen on socket
serverinfd=41
serveroutfd=42
exec 41> ${serverin}
exec 42< ${serverout}
debug server "Input pipe is now fd ${serverinfd}, output pipe is now fd ${serveroutfd}"

if ! testfiles/wait_until_listening.sh $server_pid $port ; then
    debug advice "Server not listening as expected - aborting test"
    exit 23
fi

# Create the named pipe files for communicating with the client

# Start up the client in the background
debug advice "Starting client in the background as follows:"
debug advice ${clientProg} "$@" ${port} "< ${clientin} > ${clientout} 2> ${clienterr}"
${clientProg} "$@" ${port} < ${clientin} > ${clientout} 2> ${clienterr} 41>&- 42>&- &
client_pid=$!
client_processid=${client_pid}
client_started=1

# Open the pipes
exec {clientinfd}> ${clientin}
exec {clientoutfd}< ${clientout}
debug client "Input pipe is now fd ${clientinfd}, output pipe is now fd ${clientoutfd}"

declare -i linenum=0;
# Read each line in the operations file
while read -r who request mesg <&3 ; do
    linenum+=1
    debug sequence "Time $(date +%S.%N) - Read line from sequence file: $who $request $mesg"
    if [[ $who =~ ^#.*$ ]] ; then
	# Skip over comments
	debug advice "Skipping comment"
	continue;
    fi

    case $who in 
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
	sigpipe )
            if ! is_alive $client_processid ; then 
                echo "Client has died unexpectedly - aborting" >&${errfd}
                exit 23
            fi
	    debug advice "Sending SIGPIPE to client"
	    kill -PIPE $client_pid
	    sleep 0.03
	    continue
	    ;;
        client )
            case $request in  
                expectexit )
                    sleep 0.07
                    if is_alive $client_processid ; then
                        echo "Client is alive but not expected to be - aborting" >&${errfd}
                        exit 23
                    else
                        debug client "Client has exited as expected"
                        wait ${client_pid} >&/dev/null
                        client_exit_status=$?
                        continue
                    fi
                    ;;
                close )
                    # Close the file descriptor into the client
                    eval "exec ${clientinfd}>&-" 
                    continue
                    ;;
            esac
            # Make sure client is still alive
            if ! is_alive $client_processid ; then 
                echo "Client has died unexpectedly - aborting" >&${errfd}
                exit 23
            fi
            in=${clientinfd}
            out=${clientoutfd}
            pid=${client_processid}
            ;;
        server )
            # Make sure server is still alive
            if ! is_alive ${server_processid} ; then
                echo "Server has died unexpectedly - aborting" >&${errfd}
                exit 23
            fi
            if [ ${request} = "close" ] ; then
                kill -9 ${server_processid}
                continue;
            fi
            in=${serverinfd}
            out=${serveroutfd}
            pid=${server_processid}
            ;;
	* )
            unknown_request $who $request $mesg
            # Does not return
	    ;;
    esac

    case "${request}" in
	read )
            if ! is_alive ${pid} ; then
                debug ${who} "${who} is dead - can't read line"
                echo "${who} has exited - can't read line" >> /tmp/csse2310.${who}.stdout.$$
                continue
            fi
	    unset line
	    if [ "$mesg" ] ; then
		timeout="${mesg}"
	    else
		timeout=0.2
	    fi
	    debug ${who} "Attempting to read line from ${who}'s stdout"
	    IFS=""
	    if read -t ${timeout} -r line <&${out} ; then
		# Save the line to the output file
		debug ${who} "${who} output line '${line}'"
		echo "${line}" >> /tmp/csse2310.${who}.stdout.$$
	    else
		# Got timeout which was not expected
		debug ${who} "Unexpected timeout (${timeout}s) waiting for ${who} to output line"
		echo "Got unexpected timeout (${timeout}s) waiting for line from ${who}" >> /tmp/csse2310.${who}.stdout.$$
	    fi
	    unset IFS
	    continue
	    ;;
	readtimeout )
	    unset line
	    if [ "$mesg" ] ; then
		timeout="${mesg}"
	    else
		timeout=0.1
	    fi
	    IFS=""
	    # We open pipe for reading and writing here so there is no block
	    if read -t "${timeout}" -r line <&${out} ; then
		# we expected no data but got some
		debug ${who} "Expected ${who} to output nothing in the next ${timeout}s but got line '${line}'"
		# Save the line to the output file
		echo "Expected ${timeout}s timeout on read, but got unexpected: ${line}" >> /tmp/csse2310.${who}.stdout.$$
	    else
		debug ${who} "${who} output nothing in the next ${timeout}s (as expected)"
		echo "Waited ${timeout}s - nothing arrived to ${who} (as expected)" >> /tmp/csse2310.${who}.stdout.$$
	    fi
	    unset IFS
	    continue
	    ;;
	send )
            if is_alive ${pid} ; then
                debug ${who} "Sending '${mesg}' to ${who}'s stdin"
                echo -en "${mesg}" >&${in}
            else
                debug ${who} "Can't send '${mesg}' to ${who}'s stdin - client is dead"
            fi
	    ;;
	*) 
            unknown_request $who $request $mesg
            # Does not return
	    ;;
    esac
    sleep "${delay}"
done

# Have now completed the operations - kill off processes and remove pipes
debug advice "Sequence file finished - terminating remaining processes"
terminate_processes

if [ -s /tmp/csse2310.server.stdout.$$ ] ; then
    echo "Messages received by server:"
    echo "------------------------"
    cat /tmp/csse2310.server.stdout.$$
    echo "------------------------"
fi

# Send the output from the clients to stdout
if [ -s /tmp/csse2310.client.stdout.$$ ] ; then
    echo "Output from client is:"
    echo "------------------------"
    cat /tmp/csse2310.client.stdout.$$
    echo "------------------------"
fi

# This will cause the cleanup routine above to run
exit 0
