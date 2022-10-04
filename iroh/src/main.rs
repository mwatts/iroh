use std::path::PathBuf;

use clap::{arg, Command};
use iroh::constants;

fn cli() -> Command<'static> {
    Command::new("iroh")
        .about("A next generation IPFS implementation: https://iroh.computer")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("add")
                .about("Add a file to IPFS")
                .after_help(constants::ADD_AFTER_TEXT)
                .arg(arg!(<PATH> "The path to a file to be added to IPFS"))
                .args(vec![
                    arg!(-r --recursive "Add directory paths recursively. Default: true"),
                    arg!( --"stdin-name" ... "Assign a name if the file source is stdin")
                        .required(false),
                    arg!(-H --hidden "Include files that are hidden. Only takes effect on recursive add.")
                        .required(false),
                    arg!(-p --progress "Stream progress data. Default: true")
                        .required(false),
                    arg!(-n --"only-hash" "Only chunk and hash. Do not write to disk.")
                        .required(false),
                    arg!(-w --"wrap-with-directory" "Wrap files with a directory object. Default: true"),
                    // we aren't going to do anything about pinning this release
                    // arg!(--pin "Pin this object when adding. Default: true."),
                        // .default_value(true)
                ])
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("get things")
                .arg(arg!(<CID> ... "CID to get"))
                .arg(arg!(-o --output [OUTPUT] "The path where the output should be stored.").value_parser(clap::value_parser!(PathBuf))
                        .required(false))
                .after_help(constants::GET_AFTER_TEXT)
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("p2p")
                .about("peer-2-peer commands")
                .subcommand_required(true)
                .after_help(constants::P2P_AFTER_TEXT)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("id")
                    .about("show IPFS node id info")
                    .after_help(constants::P2P_ID_AFTER_TEXT)
                )
                .subcommand(
                    Command::new("connect")
                        .about("connect to a peer")
                        .arg(arg!(<ADDRESS> ... "address of a peer to connect to"))
                        .after_help(constants::P2P_CONNECT_AFTER_TEXT)
                        .arg_required_else_help(true),
                )
        )
        .subcommand(
            Command::new("start")
                .about("start a long running IPFS process")
                .after_help(constants::START_AFTER_TEXT)
        )
        .subcommand(
            Command::new("status")
                .about("report current status of iroh")
                .arg(arg!(-w --watch "poll process for changes"))
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