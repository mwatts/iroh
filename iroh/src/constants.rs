

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


pub static GET_AFTER_TEXT: &str = "";
pub static ID_AFTER_TEXT: &str = "";
pub static P2P_AFTER_TEXT: &str = "";
pub static P2P_CONNECT_AFTER_TEXT: &str = "";
pub static P2P_DISCONNECT_AFTER_TEXT: &str = "";
pub static START_AFTER_TEXT: &str = "";
pub static STATUS_AFTER_TEXT: &str = "";
pub static VERSION_AFTER_TEXT: &str = "";