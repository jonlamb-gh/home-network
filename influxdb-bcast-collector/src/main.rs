use log::Level;
use std::net::SocketAddr;
use structopt::StructOpt;

// TODO - user/pass for auth
#[derive(Debug, StructOpt)]
#[structopt(about = "TODO Todo.")]
struct CLIOptions {
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// UDP address:port
    #[structopt(short = "a", long, default_value = "0.0.0.0:9876")]
    address: SocketAddr,

    /// Database client address:port
    #[structopt(short = "c", long, default_value = "http://localhost:8086")]
    client: String,

    /// Database name
    #[structopt(short = "d", long, default_value = "parameters")]
    database: String,
}

fn main() {
    let opts = CLIOptions::from_args();

    if opts.verbose {
        simple_logger::init_with_level(Level::Info).unwrap();
    } else {
        simple_logger::init_with_level(Level::Warn).unwrap();
    }

    influxdb_bcast_collector::start_listening(opts.address, opts.client, opts.database).unwrap();
}
