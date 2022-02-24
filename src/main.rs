#![type_length_limit = "2097152"]
#[macro_use]
mod macros;
mod args;
use bollard::Docker;
use containers::{check_not_running_containers, check_running_containers};

// use crate::args::{Args, MyCommand};
use crate::args::{Args, Command};
mod containers;

static mut VERBOSE: bool = false;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();
    match args.verbose.log_level() {
        None => unsafe {
            VERBOSE = false;
        },
        Some(_) => unsafe {
            VERBOSE = true;
        },
    }
    printlnv!("Args are {:?}.", args);
    match &args.command {
        Command::Print {} => {
            let docker = Docker::connect_with_socket_defaults().unwrap();
            check_running_containers(&docker).await?;
            check_not_running_containers(&docker, &args.label).await?;
        }
        Command::NotifyTeams { .. } => println!("Not implemented."),
        Command::NotifyWebhook { .. } => println!("Not implemented."),
    }
    Ok(())
}
