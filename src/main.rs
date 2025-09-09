mod args;
mod commands;
mod events;

use args::ARGS;
use log::{debug, error, info, trace};
mod nixpkgs;

use crate::nixpkgs::NixpkgsRepo;
use poise::FrameworkOptions;
use poise::serenity_prelude::Client;
use poise::{Command, Framework, serenity_prelude as serenity};
use serenity::prelude::*;
use std::error;

type Error = Box<dyn error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    // Just a heads up, before init() is called on colog, our logging library,
    // do not expect any form of logging to work.
    // This notice should rarely matter, but there are instances in early runtime where it may.
    // During CLI args evaluation is a good example.
    init_logging();

    // Let's go ahead and spawn a thread to clone nixpkgs, it will take a minute.
    tokio::spawn(async move {
        let nixpkgs = NixpkgsRepo.lock().await;
        info!("Nixpkgs is ready at: {}", nixpkgs.path().display());
    });

    let mut client: Client = build_client(&ARGS.token).await;
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
    let commands: Vec<Command<(), Error>> =
        vec![commands::ping(), commands::eval(), commands::maintainer()];
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

pub(crate) fn init_logging() {
    // Before now, logging is unavailable, therefore we may not log yet.
    colog::default_builder()
        .filter(None, ARGS.dependency_log_level)
        .filter(Some(env!("CARGO_CRATE_NAME")), ARGS.log_level)
        .init();
    // Now, we may begin logging.
    debug!("Logging is ready!");
}
