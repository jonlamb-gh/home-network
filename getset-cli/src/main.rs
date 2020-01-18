use log::Level;
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "TODO Todo.")]
struct CLIOptions {
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    // TODO - get and set subcmd's
    Listen {
        /// UDP address:port
        #[structopt(short = "a", long, default_value = "0.0.0.0:9876")]
        address: SocketAddr,
    },
    ListAll {
        /// TCP address:port
        #[structopt(short = "a", long, default_value = "192.168.1.39:9877")]
        address: SocketAddr,
    },
}

fn main() {
    let opts = CLIOptions::from_args();

    if opts.verbose {
        simple_logger::init_with_level(Level::Trace).unwrap();
    } else {
        simple_logger::init_with_level(Level::Warn).unwrap();
    }

    match opts.cmd {
        Command::Listen { address } => getset_cli::start_listening(address).unwrap(),
        Command::ListAll { address } => getset_cli::list_all(address).unwrap(),
    }
}
