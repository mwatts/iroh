use std::path::PathBuf;

use clap::{arg, Command};
use iroh::constants;

fn cli() -> Command<'static> {
    Command::new("iroh")
        .about("A next generation IPFS implementation: https://iroh.computer")
        .subcommand_required(true)
        .arg(arg!(-v --version "print iroh version information"))
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("add")
                .about("Add a file to iroh & make it available on IPFS")
                .after_help(constants::ADD_AFTER_TEXT)
                .arg(arg!(<PATH> "The path to a file or directory to be added"))
                .args(vec![
                    arg!(-r --recursive "Add directory paths recursively. Default: false")
                        .required(false),
                    arg!(-H --hidden "Include files that are hidden. Only takes effect on recursive add.")
                        .required(false),
                    arg!(-p --progress "Stream progress data. Default: true")
                        .required(false),
                    arg!(-n --"only-hash" "Only chunk and hash. Do not write to disk.")
                        .required(false),
                    arg!(-w --"wrap-with-directory" "Wrap files with a directory object. Default: true"),
                    arg!(-C --"root-cid" "Output only the final Content Identifier (CID)"),
                    // we aren't going to do anything about pinning this release
                    // arg!(--pin "Pin this object when adding. Default: true."),
                        // .default_value(true)
                ])
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("Fetch IPFS content and write it to disk")
                .arg(arg!(<"ipfs-path"> "CID or CID/with/path/qualifier to get"))
                .arg(arg!([output] "filesystem path to write to. Default: $CID").value_parser(clap::value_parser!(PathBuf))
                        .required(false))
                .arg(arg!(--"force-fetch" "ignore local cache & fetch all content from the network")
                    .required(false))
                .after_help(constants::GET_AFTER_TEXT)
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("p2p")
                .about("Peer-2-peer commands")
                .subcommand_required(true)
                .after_help(constants::P2P_AFTER_TEXT)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("lookup")
                    .about("Retrieve info about a peer")
                    .arg(arg!(<addr> "multiaddress or peer ID"))
                    .after_help(constants::P2P_LOOKUP_AFTER_TEXT)
                )
                .subcommand(
                    Command::new("connect")
                        .about("Connect to a peer")
                        .arg(arg!(<ADDRESS> ... "address of a peer to connect to"))
                        .after_help(constants::P2P_CONNECT_AFTER_TEXT)
                        .arg_required_else_help(true),
                )
        )
        .subcommand(
            Command::new("start")
                .about("Start a long running IPFS process")
                .after_help(constants::START_AFTER_TEXT)
        )
        .subcommand(
            Command::new("status")
                .about("Report current status of iroh")
                .arg(arg!(-w --watch "Poll process for changes"))
                .after_help(constants::STATUS_AFTER_TEXT)
        )
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            println!(
                "Adding {}",
                sub_matches.get_one::<String>("REMOTE").expect("required")
            );
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}