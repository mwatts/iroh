
pub static IROH_AFTER_TEXT: &str = "
IROH_PATH environment variable

ipfs uses a repository in the local file system. By default, the repo is
located at ~/.ipfs. To change the repo location, set the $IROH_PATH
environment variable:

  export IROH_PATH=/path/to/ipfsrepo
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

Once content is added to iroh, it can be provided & rehosted by any other IPFS
node with iroh start:

  > iroh start

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
Attempts to open a new direct connection to a peer address.

The address format is in multiaddr format

  > iroh p2p connect /ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ

If only a peer identifier is provided, a DHT lookup

for more info, see https://iroh.computer/docs/concepts#multiaddr
";

pub static START_AFTER_TEXT: &str = "
start kicks off a long running process that manages p2p connections, serves any 
configured APIs, and proxies access for other iroh processes to shared resources
like store and network connections. While iroh start is running iroh will 
initiate connections to other peers in the network to both provide and fetch 
content, all of which can be governed through configuration.

When iroh start is running, it acquires a lock on shared resources. Running an
iroh command from another terminal will be executed as a remote procedure call
on the 'iroh start' process. To check if a command will run through 'iroh start'
use `iroh status`.

Shutdown

To stop iroh start, send a SIGINT signal to it (e.g. by pressing 'Ctrl-C')
or send a SIGTERM signal to it (e.g. with 'kill'). It may take a while for the
daemon to shutdown gracefully, but it can be killed forcibly by sending a
second signal.
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
