

pub static ADD_AFTER_TEXT: &str = "
Adds the content of <path> to IPFS. Use -r to add directories.
  Note that directories are added recursively, to form the IPFS
  MerkleDAG.

  If the daemon is not running, it will just add locally.
  If the daemon is started later, it will be advertised after a few
  seconds when the reprovider runs.

  The wrap option, '-w', wraps the file (or files, if using the
  recursive option) in a directory. This directory contains only
  the files which have been added, and means that the file retains
  its filename. For example:

    > ipfs add example.jpg
    added QmbFMke1KXqnYyBBWxB74N4c5SBnJMVAiMNRcGu6x1AwQH example.jpg
    > ipfs add example.jpg -w
    added QmbFMke1KXqnYyBBWxB74N4c5SBnJMVAiMNRcGu6x1AwQH example.jpg
    added QmaG4FuMqEBnQNn3C8XJ5bpW8kLs7zq2ZXgHptJHbKDDVx

  You can now refer to the added file in a gateway, like so:

    /ipfs/QmaG4FuMqEBnQNn3C8XJ5bpW8kLs7zq2ZXgHptJHbKDDVx/example.jpg

  The chunker option, '-s', specifies the chunking strategy that dictates
  how to break files into blocks. Blocks with same content can
  be deduplicated. Different chunking strategies will produce different
  hashes for the same file. The default is a fixed block size of
  256 * 1024 bytes, 'size-262144'. Alternatively, you can use the
  Buzhash or Rabin fingerprint chunker for content defined chunking by
  specifying buzhash or rabin-[min]-[avg]-[max] (where min/avg/max refer
  to the desired chunk sizes in bytes), e.g. 'rabin-262144-524288-1048576'.

  The following examples use very small byte sizes to demonstrate the
  properties of the different chunkers on a small file. You'll likely
  want to use a 1024 times larger chunk sizes for most files.

    > ipfs add --chunker=size-2048 ipfs-logo.svg
    added QmafrLBfzRLV4XSH1XcaMMeaXEUhDJjmtDfsYU95TrWG87 ipfs-logo.svg
    > ipfs add --chunker=rabin-512-1024-2048 ipfs-logo.svg
    added Qmf1hDN65tR55Ubh2RN1FPxr69xq3giVBz1KApsresY8Gn ipfs-logo.svg

  You can now check what blocks have been created by:

    > ipfs object links QmafrLBfzRLV4XSH1XcaMMeaXEUhDJjmtDfsYU95TrWG87
    QmY6yj1GsermExDXoosVE3aSPxdMNYr6aKuw3nA8LoWPRS 2059
    Qmf7ZQeSxq2fJVJbCmgTrLLVN9tDR9Wy5k75DxQKuz5Gyt 1195
    > ipfs object links Qmf1hDN65tR55Ubh2RN1FPxr69xq3giVBz1KApsresY8Gn
    QmY6yj1GsermExDXoosVE3aSPxdMNYr6aKuw3nA8LoWPRS 2059
    QmerURi9k4XzKCaaPbsK6BL5pMEjF7PGphjDvkkjDtsVf3 868
    QmQB28iwSriSUSMqG2nXDTLtdPHgWb4rebBrU7Q1j4vxPv 338

  Finally, a note on hash determinism. While not guaranteed, adding the same
  file/directory with the same flags will almost always result in the same output
  hash. However, almost all of the flags provided by this command (other than pin,
  only-hash, and progress/status related flags) will change the final hash.
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