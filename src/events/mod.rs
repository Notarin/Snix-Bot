use crate::Error;
use log::trace;
use poise::serenity_prelude::{Context, FullEvent};

pub(crate) mod ready;

pub(crate) async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, (), Error>,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => ready::ready(ctx, data_about_bot).await,
        _ => trace!("Got unhandled event: {}", event.snake_case_name()),
    }
    Ok(())
}
