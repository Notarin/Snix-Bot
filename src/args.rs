use clap::Parser;
use log::LevelFilter;
use std::sync::LazyLock;

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
    #[clap(
        short,
        long,
        env,
        default_value = "https://github.com/NixOS/nixpkgs",
        help = "URL for the nixpkgs repo to clone."
    )]
    pub(crate) nixpkgs_url: String,
    #[clap(
        short,
        long,
        env,
        default_value = "1",
        help = "Clone depth for the nixpkgs repo."
    )]
    pub(crate) clone_depth: i32,
}

pub(crate) static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);
