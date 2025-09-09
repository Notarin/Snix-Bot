use clap::Parser;
use lazy_static::lazy_static;
use log::{LevelFilter, debug};

#[derive(Parser)]
#[command(version, about, author)]
pub(crate) struct Args {
    #[clap(
        short,
        long,
        env,
        help = "The authentication token for logging into the Discord bot account."
    )]
    pub(crate) token: String,
    #[clap(
        short,
        long,
        env,
        default_value = "Info",
        help = "Logging level for the bot crate alone."
    )]
    pub(crate) log_level: LevelFilter,
    #[clap(
        short,
        long,
        env,
        default_value = "Warn",
        help = "Logging level for all crates other than the bot itself."
    )]
    pub(crate) dependency_log_level: LevelFilter,
}

lazy_static! {
    pub(crate) static ref ARGS: Args = {
        // No logging takes place here as colog, our logging library, depends on these args,
        // meaning we cannot possibly have logging ready.
        // See the comment (which hopefully is there! XD) at the top of main() for more info.
        Args::parse()
    };
}

pub(crate) fn init_logging() {
    // Before now, logging is unavailable, therefore we may not log yet.
    colog::default_builder()
        .filter(None, ARGS.dependency_log_level)
        .filter(Some(env!("CARGO_CRATE_NAME")), ARGS.log_level)
        .init();
    // Now, we may begin logging.
    debug!("Logging is ready!");
}
