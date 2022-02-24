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

    #[clap(subcommand)]
    pub command: Command,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Prints the status to stdout.")]
    Print {},
    #[clap(about = "Sends a notification through Microsoft Teams")]
    NotifyTeams {
        #[clap(short, long, help = "Teams callback url")]
        callback_url: String,
    },
    #[clap(about = "Sends a notification through a webhook")]
    NotifyWebhook {
        #[clap(short, long, help = "Webhook url")]
        callback_url: String,
    },
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_run_simulated() {
        let args = Args::new_from(["notifyhealth", "--label", "foo", "print"].iter());
        match args.command {
            Command::NotifyTeams { .. } => panic!("Should not be notify teams"),
            Command::NotifyWebhook { .. } => panic!("Should not notify webhook"),
            Command::Print {} => (),
        };
        assert_eq!("foo", args.label);
    }
}
