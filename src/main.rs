#![type_length_limit = "2097152"]
#![warn(clippy::shadow_unrelated)]
#[macro_use]
mod macros;
mod args;
mod print;
use crate::{
    args::{Args, Command},
    containers::Containers,
};
use bollard::Docker;
use containers::{check_not_running_containers, check_running_containers};
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
            let containers = Containers::new(docker);
            let running_containers = check_running_containers(&containers).await?;
            print::running_containers(running_containers);
            let stopped_containers = check_not_running_containers(&containers, &args.label).await?;
            print::stopped_containers(stopped_containers);
        }
        Command::NotifyTeams { .. } => println!("Not implemented."),
        Command::NotifyWebhook { .. } => println!("Not implemented."),
    }
    Ok(())
}
