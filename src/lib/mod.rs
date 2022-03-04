#![warn(clippy::shadow_unrelated)]
#[macro_use]
mod macros;
pub mod args;
pub mod containers;
pub mod print;
pub mod webhook;
use args::*;
use bollard::Docker;
use containers::Containers;
use log::info;
use log::{Level, LevelFilter};
use webhook::Webhook;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();
    let level = to_level_filter(args.verbose.log_level());
    env_logger::Builder::new().filter_level(level).init();
    info!("Log level: {level}");
    info!("Args are {:?}.", args);
    match &args.command {
        Command::Print {} => {
            let docker = Docker::connect_with_socket_defaults().unwrap();
            let containers = Containers::new(docker);
            let running_containers = containers::check_running_containers(&containers).await?;
            print::running_containers(running_containers);
            let stopped_containers = containers::check_not_running_containers(&containers, &args.label).await?;
            print::stopped_containers(stopped_containers);
        }
        Command::NotifyTeams { .. } => println!("Not implemented."),
        Command::NotifyWebhook { callback_url } => {
            let docker = Docker::connect_with_socket_defaults().unwrap();
            let containers = Containers::new(docker);
            let running_containers = containers::check_running_containers(&containers).await?;
            let stopped_containers = containers::check_not_running_containers(&containers, &args.label).await?;
            Webhook::shared().notify(callback_url, running_containers, stopped_containers)?;
        }
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