
pub static IROH_AFTER_TEXT: &str = "
Iroh is a next-generation implementation the Interplanetary File System (IPFS). 
IPFS is a networking protocol for exchanging content-addressed blocks of 
immutable data. 'content-addressed' means referring to data by the hash of it's
content, which makes the reference both unique and verifiable. These two 
properties make it possible to get data from any node in the network that speaks
the IPFS protocol, including IPFS content being served by other implementations
of the protocol.

For more info see https://iroh.computer/docs
";

pub static ADD_AFTER_TEXT: &str = "
NOTE: IROH CURRENTLY PROVIDES NO WAY TO REMOVE CONTENT ONCE ADDED. This will be
addressed in a future release.

Add copies the file or directory specified by of <path> into the iroh store, 
splitting the input file into a tree of immutable blocks. Each block is labeled 
by the hash of its content. The final output of the add command is the hash of 
the root of the tree, which contains references to all other blocks:

  > iroh add example.jpg
  added [HASH] example.jpg
  added [ROOT_CID]

[ROOT_CID] is a Content IDentifier (or CID), which is a hash plus some extra
metadata. The opposite of the add command is the get command, which accepts a 
CID and turns it back into files or directories:

  > iroh get [ROOT_CID]
  [ROOT_CID] written to working directory

The stored result of add is a 'MerkleDAG'. Merkle proofs (hashes) are a fast 
method of proving and checking data inclusion, and the tree formed by chunking 
the input into blocks is always a directed acyclic graph (DAG). These MerkleDAGs
can be provably checked for tamper resistance by anyone who fetches all blocks 
in the tree, which means MerkleDAGs can be provided by anyone, without concern 
for tampering.

By default all content added to iroh is made available to the network, and the
default network is the public IPFS network. We can prove this by creating an
empty iroh instance. Run this in a separate terminal with iroh start running:

  > IROH_PATH=iroh_temp && iroh get [ROOT_CID]

We can use an HTTPS gateway hosted at https://gateway.lol to fetch the content 
from the node running with iroh start:

  > curl https://gateway.lol/ipfs/[ROOT_CID]/example.jpg

In this case we're trusting whoever runs gateway.lol to verify the merkle proof
for us, because the last mile of our request is over HTTPS, whereas 'iroh get'
always performs this check locally.

The wrap option, '-w', wraps the file (or files, if using the recursive option) 
in a directory and is on by default. This directory contains only the files 
which have been added so the input retains its filename. set -w=false for
'raw' adds that lose their input name:

    > iroh add example.jpg
    added [CID] example.jpg
    added [ROOT_CID]
    > iroh add example.jpg -w=false
    added [ROOT_CID] example.jpg

Note that in both cases the CID of example.jpg is the same. Adding the same 
file/directory with the same flags is deterministic, which means the same input 
paired with the same configuration flags will give the same root hash.
";

pub static GET_AFTER_TEXT: &str = "
Download file or directory specified by <ipfs-path> from IPFS into [path]. If 
path already exists and is a file then it's overwritten with the new downloaded 
file. If path already exists and is a directory, the command fails with an 
error. If path already exists, is a file and the downloaded data is a directory,
that's an error.

By default, the output will be written to './<ipfs-path>'.

If <ipfs-path> is already present in the iroh store, no network call will 
be made.
";

pub static P2P_LOOKUP_AFTER_TEXT: &str = "
Takes as input a peer ID or address and prints the output of the libp2p-identify
protocol. When provided with a peer ID, the address is looked up on the 
Network's Distribted Hash Table (DHT) before connecting to the node. When 
provided with a multiaddress, the connection dialed directly.
";

pub static P2P_AFTER_TEXT: &str = "
p2p commands all relate to peer-2-peer connectivity. See subcommands for
additional details.";

pub static P2P_CONNECT_AFTER_TEXT: &str = "
Attempts to open a new direct connection to a peer address. By default p2p 
continulously maintains an open set of peer connections based on requests &
internal hueristics. Connect is useful in situations where it makes sense to
manually force libp2p to dial a known peer. A common example includes when you
know the multiaddr or peer ID of a peer that you would like to exchange data 
with.

The address format is in multiaddr format. For example:

  > iroh p2p connect /ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ

for more info on multiaddrs see https://iroh.computer/docs/concepts#multiaddr

If a peer ID is provided, connect first perform a distribtued hash table (DHT)
lookup to learn the address of the given peer ID before dialing.
";

pub static START_AFTER_TEXT: &str = "
start kicks off one or more 'deamon' processes. An iroh deamon is a long running
process that both handle requests and initiate required background work. Iroh 
requires a running deamon to do anything meaningful.

The default daemon is iroh-one, a single executable that provides gateway, p2p 
and store services. iroh start can be configured to run each service as
separate processes if desired. See https://iroh.computer/docs/configuration for
details.

Check the current status of the iroh daemon with 'iroh status', stop the iroh
deamon with 'iroh stop'. See help text of the status & stop commands for more
details.

Start is just a convenience shortcut for initiating the deamon process. It's 
equivelant to running ./iroh-one from a terminal, and in many cases manually
starting iroh daemons is preferrable. For example: when scheduleing via a
service manager like systemd on linux, or when running iroh in the cloud via 
DevOps platforms like Ansible or Kubernetes.

When a deamon is running, it acquires a lock on shared resources. Only one 
service can be running at a time with the same lock.
";

pub static STOP_AFTER_TEXT: &str = "
stop checks for a running iroh daemon and halts any daemon process it finds.
stop sends SIGINT to each process and waits for the process to exit gracefully.
If the process does not exit gracefully on it's own within 30 seconds stop sends
SIGKILL to the deamon process and exits.

If stop finds no running processes it does nothing and exits with status code 0.

Processes are identified via their process ID as recorded on the local program
lock. If stop cannot find lock files on the local filesystem (possibly because
iroh is configured to use a network-backed deamon), it will exit with an error.
";

pub static STATUS_AFTER_TEXT: &str = "
status reports the current operational setup of iroh. Use status as a go-to
command for understanding where iroh commands are being processed. different
ops configurations utilize different network and service implementations 
under the hood, which can lead to varying performance characteristics.

Status reports connectivity, which is either offline or online:

  offline: iroh is not connected to any background process, all commands 
           are one-off, any network connections are closed when a command
           completes. Some network duties may be delegated to remote hosts.

  online:  iroh has found a long-running process to issue commands to. Any
           comand issued will be deletegated to the long-running process as a
           remote procedure call

If iroh is online, status also reports the service configuration of the
long running process, including the health of the configured subsystem(s).
Possible configurations fall into two buckets:

  one:     Iroh is running with all services bundled into one single process,
           this setup is common in desktop enviornments.

  cloud:   Iroh is running with services split into separate processes, which
           are speaking to each other via remote procedure calls.

Use the --watch flag to continually poll for changes.

Status reports no metrics about the running system aside from current service
health. Instead all metrics are emitted through uniform tracing collection &
reporting, which is intended to be consumed by tools like prometheus and 
grafana. For more info on metrics collection, see 
https://iroh.computer/docs/metrics
";
