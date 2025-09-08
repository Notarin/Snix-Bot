mod commands;
mod events;
use clap::Parser;
use log::{LevelFilter, error, info, trace};
use poise::FrameworkOptions;
use poise::serenity_prelude::Client;
use poise::{Command, Framework, serenity_prelude as serenity};
use serenity::prelude::*;
use std::error;

#[derive(Parser)]
#[command(version, about, author)]
struct Args {
    #[clap(
        short,
        long,
        env,
        help = "The authentication token for logging into the Discord bot account."
    )]
    token: String,
    #[clap(
        short,
        long,
        env,
        default_value = "Info",
        help = "Logging level for the bot crate alone."
    )]
    log_level: LevelFilter,
    #[clap(
        short,
        long,
        env,
        default_value = "Warn",
        help = "Logging level for all crates other than the bot itself."
    )]
    dependency_log_level: LevelFilter,
}

type Error = Box<dyn error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    let Args {
        token,
        log_level,
        dependency_log_level,
    }: Args = Args::parse();
    colog::default_builder()
        .filter(None, dependency_log_level)
        .filter(Some(env!("CARGO_CRATE_NAME")), log_level)
        .init();

    let mut client: Client = build_client(&token).await;
    info!("Starting client.");
    let result: serenity::Result<()> = client.start().await;
    info!("Client has shut down, finishing up.");
    match result {
        Ok(()) => {
            info!("Client undergoing graceful shutdown.")
        }
        Err(error) => {
            error!("Client error causing full panic: {error:?}");
        }
    }
}

async fn build_client(token: &String) -> Client {
    let framework: Framework<(), Error> = build_framework();

    let intents: GatewayIntents = GatewayIntents::all();
    trace!("Building client.");
    let client: Client = Client::builder(token, intents)
        .framework(framework)
        .await
        .expect("Failed to build client!");
    client
}

fn build_framework() -> Framework<(), Error> {
    trace!("Collecting commands.");
    let commands: Vec<Command<(), Error>> = vec![commands::ping(), commands::eval()];
    trace!("Building bot framework.");
    let framework_options = FrameworkOptions {
        commands,
        event_handler: |ctx, event, framework, _data| {
            Box::pin(events::event_handler(ctx, event, framework))
        },
        ..Default::default()
    };
    let framework: Framework<(), Error> = Framework::builder()
        .options(framework_options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();
    framework
}
