

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
Stores to disk the data contained an IPFS or IPNS object(s) at the given path.

By default, the output will be stored at './<ipfs-path>', but an alternate
path can be specified with '--output=<path>' or '-o=<path>'.

To output a TAR archive instead of unpacked files, use '--archive' or '-a'.

To compress the output with GZIP compression, use '--compress' or '-C'. You
may also specify the level of compression by specifying '-l=<1-9>'.";

pub static P2P_ID_AFTER_TEXT: &str = "
Prints out information about the specified peer.
If no peer is specified, prints out information for local peers.

'ipfs id' supports the format option for output with the following keys:
<id> : The peers id.
<aver>: Agent version.
<pver>: Protocol version.
<pubkey>: Public key.
<addrs>: Addresses (newline delimited).

EXAMPLE:

    ipfs id Qmece2RkXhsKe5CRooNisBTh4SK119KrXXGmoK6V3kb8aH -f=\"<addrs>
";

pub static P2P_AFTER_TEXT: &str = "";

pub static P2P_CONNECT_AFTER_TEXT: &str = "'ipfs swarm connect' opens a new direct connection to a peer address.

The address format is an IPFS multiaddr:

ipfs swarm connect /ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ";

pub static START_AFTER_TEXT: &str = "
The daemon will start listening on ports on the network, which are
documented in (and can be modified through) 'ipfs config Addresses'.
For example, to change the 'Gateway' port:

  ipfs config Addresses.Gateway /ip4/127.0.0.1/tcp/8082

The API address can be changed the same way:

  ipfs config Addresses.API /ip4/127.0.0.1/tcp/5002

Make sure to restart the daemon after changing addresses.

By default, the gateway is only accessible locally. To expose it to
other computers in the network, use 0.0.0.0 as the ip address:

  ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080

Be careful if you expose the API. It is a security risk, as anyone could
control your node remotely. If you need to control the node remotely,
make sure to protect the port as you would other services or database
(firewall, authenticated proxy, etc).

Shutdown

To shut down the daemon, send a SIGINT signal to it (e.g. by pressing 'Ctrl-C')
or send a SIGTERM signal to it (e.g. with 'kill'). It may take a while for the
daemon to shutdown gracefully, but it can be killed forcibly by sending a
second signal.

IPFS_PATH environment variable

ipfs uses a repository in the local file system. By default, the repo is
located at ~/.ipfs. To change the repo location, set the $IPFS_PATH
environment variable:

  export IPFS_PATH=/path/to/ipfsrepo
";
pub static STATUS_AFTER_TEXT: &str = "";
pub static VERSION_AFTER_TEXT: &str = "";