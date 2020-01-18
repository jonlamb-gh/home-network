use log::Level;
use params::{ParameterValue, ParameterValueTypeId};
use std::net::SocketAddr;
use std::str::FromStr;
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
    /// Listen for broadcast parameters
    Listen {
        /// UDP address:port
        #[structopt(short = "a", long, default_value = "0.0.0.0:9876")]
        address: SocketAddr,
    },

    /// List all parameters
    ListAll {
        /// TCP address:port
        #[structopt(short = "a", long, default_value = "192.168.1.39:9877")]
        address: SocketAddr,
    },

    /// Get parameter(s) by ID
    Get {
        /// TCP address:port
        #[structopt(short = "a", long, default_value = "192.168.1.39:9877")]
        address: SocketAddr,

        /// Parameter ID
        #[structopt(short = "i", long)]
        id: u32,
    },

    /// Set parameter(s) by ID, value
    Set {
        /// TCP address:port
        #[structopt(short = "a", long, default_value = "192.168.1.39:9877")]
        address: SocketAddr,

        /// Parameter ID
        #[structopt(short = "i", long)]
        id: u32,

        /// Parameter value type
        #[structopt(short = "t", long = "type", parse(try_from_str = parse_value_type))]
        value_type: ParameterValueTypeId,

        /// Parameter value
        #[structopt(short = "v", long)]
        value: String,
    },
}

fn parse_value_type(src: &str) -> Result<ParameterValueTypeId, String> {
    match ParameterValueTypeId::from_str(src) {
        Ok(t) => Ok(t),
        _ => Err(format!("Invalid ParameterValueTypeId: {}", src)),
    }
}

// TODO - lookup value type from param-desc/db instead of cli provided
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
        Command::Get { address, id } => getset_cli::get(address, id.into()).unwrap(),
        Command::Set {
            address,
            id,
            value_type,
            value,
        } => {
            println!("type {:?}", value_type);
            let value = match value_type {
                ParameterValueTypeId::None => ParameterValue::None,
                ParameterValueTypeId::Notification => ParameterValue::Notification,
                ParameterValueTypeId::Bool => ParameterValue::Bool(bool::from_str(&value).unwrap()),
                ParameterValueTypeId::U8 => ParameterValue::U8(u8::from_str(&value).unwrap()),
                ParameterValueTypeId::U32 => ParameterValue::U32(u32::from_str(&value).unwrap()),
                ParameterValueTypeId::I32 => ParameterValue::I32(i32::from_str(&value).unwrap()),
                ParameterValueTypeId::F32 => ParameterValue::F32(f32::from_str(&value).unwrap()),
            };
            getset_cli::set(address, id.into(), value).unwrap()
        }
    }
}
