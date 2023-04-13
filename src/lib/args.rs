use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author = "Giovanni Bassi <giggio@giggio.net>", version = env!("CARGO_PKG_VERSION"), about = "Checks containers status and notifies problems", long_about = None)]
pub struct Args {
    #[clap(
        short,
        long,
        help = "Container label to check for stopped containers",
        required = true
    )]
    pub label: String,
    #[clap(long, help = "Host name of docker host")]
    pub hostname: Option<String>,
    #[clap(short, long, help = "Include running containers which have no health check")]
    pub report_no_health: bool,

    #[clap(subcommand)]
    pub command: Command,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Calls an executable.")]
    Exec {
        // #[command(flatten)]
        // exec_args: ExecArgs,
        #[arg(short, long, help = "Executable to call for running or stopped containers")]
        exec: Option<String>,
        #[arg(
            conflicts_with = "exec",
            short = 'r',
            long,
            help = "Executable to call for running containers"
        )]
        exec_running: Option<String>,
        #[arg(
            conflicts_with = "exec",
            short = 's',
            long,
            help = "Executable to call for stopped containers"
        )]
        exec_stopped: Option<String>,
    },
    #[clap(about = "Prints the status to stdout.")]
    Print {},
    #[clap(about = "Sends a notification through Microsoft Teams")]
    NotifyTeams {
        #[arg(short, long, help = "Teams callback url")]
        callback_url: String,
    },
    #[clap(about = "Sends a notification through a webhook")]
    NotifyWebhook {
        #[arg(short, long, help = "Webhook url")]
        callback_url: String,
    },
}

// #[derive(clap::Args, Debug)]
// #[group(multiple = false)]
// pub struct ExecArgs {
//     #[arg(short, long, help = "Executable to call for running or stopped containers")]
//     pub exec: Option<String>,
//     #[command(flatten)]
//     pub exec_separated: ExecSeparated,
// }

// #[derive(clap::Args, Debug)]
// #[group(multiple = true)]
// pub struct ExecSeparated {
//     #[arg(short = 'r', long, help = "Executable to call for running containers")]
//     pub exec_running: Option<String>,
//     #[arg(short = 's', long, help = "Executable to call for stopped containers")]
//     pub exec_stopped: Option<String>,
// }

impl Args {
    pub fn new() -> Args {
        Args::parse()
    }
    #[allow(dead_code)]
    fn new_from<I, T>(args: I) -> Args
    where
        I: Iterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Args::parse_from(args)
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_run_simulated() {
        let args = Args::new_from(["notifyhealth", "--label", "foo", "print"].iter());
        match args.command {
            Command::NotifyTeams { .. } => panic!("Should not be notify teams"),
            Command::NotifyWebhook { .. } => panic!("Should not notify webhook"),
            Command::Exec { .. } => panic!("Should not exec"),
            Command::Print {} => (),
        };
        assert_eq!("foo", args.label);
    }
}
