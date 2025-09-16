use crate::Context;
use log::{info, trace};
use poise::serenity_prelude::CacheHttp;
use poise::serenity_prelude::Ready;

pub(crate) async fn ready(framework: Context<'_>, data_about_bot: &Ready) {
    trace!("Received ready event.");
    info!("Logged in as {}", data_about_bot.user.name);
    let global_commands: Vec<poise::serenity_prelude::Command> = framework
        .serenity_context
        .http()
        .get_global_commands()
        .await
        .unwrap();
    info!(
        "Registered commands:\n{}",
        global_commands
            .iter()
            .map(|command| format!("- {}", command.name.clone()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
