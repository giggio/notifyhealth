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
use log::{Level, LevelFilter};
mod containers;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();
    let level = to_level_filter(args.verbose.log_level());
    env_logger::Builder::new().filter_level(level).init();
    info!("Log level: {level}");
    info!("Args are {:?}.", args);
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

fn to_level_filter(level: Option<Level>) -> LevelFilter {
    match level {
        None => LevelFilter::Off,
        Some(level) => match level {
            Level::Error => LevelFilter::Error,
            Level::Warn => LevelFilter::Warn,
            Level::Info => LevelFilter::Info,
            Level::Debug => LevelFilter::Debug,
            Level::Trace => LevelFilter::Trace,
        },
    }
}
