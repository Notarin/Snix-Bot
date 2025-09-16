use crate::{Context, Error};
use log::trace;
use poise::serenity_prelude::FullEvent;

pub(crate) mod ready;

pub(crate) async fn event_handler(framework: Context<'_>, event: &FullEvent) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => ready::ready(framework, data_about_bot).await,
        _ => trace!("Got unhandled event: {}", event.snake_case_name()),
    }
    Ok(())
}
