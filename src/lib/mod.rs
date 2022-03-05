#![warn(clippy::shadow_unrelated)]
#[macro_use]
mod macros;
pub mod args;
pub mod containers;
pub mod msteams;
pub mod print;
pub mod webhook;
use args::*;
use bollard::Docker;
use containers::Containers;
use log::{info, warn};
use log::{Level, LevelFilter};
use webhook::Webhook;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();
    let level = to_level_filter(args.verbose.log_level());
    env_logger::Builder::new().filter_level(level).init();
    info!("Log level: {level}");
    info!("Args are {:?}.", args);
    let docker = Docker::connect_with_socket_defaults().unwrap();
    let containers = Containers::new(docker);
    let running_containers = containers::check_running_containers(&containers, args.report_no_health).await?;
    warn!("Running containers: {:?}", running_containers);
    let stopped_containers = containers::check_not_running_containers(&containers, &args.label).await?;
    warn!("Stopped containers: {:?}", stopped_containers);
    match &args.command {
        Command::Print {} => {
            print::running_containers(running_containers);
            print::stopped_containers(stopped_containers);
        }
        Command::NotifyTeams { callback_url } => {
            Webhook::new(Some(msteams::format_message)).notify(
                callback_url,
                running_containers,
                stopped_containers,
                args.hostname,
            )?;
        }
        Command::NotifyWebhook { callback_url } => {
            Webhook::default().notify(callback_url, running_containers, stopped_containers, args.hostname)?;
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
