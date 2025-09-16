use crate::Error;
use poise::serenity_prelude::CreateEmbed;
use snix_eval::{EvaluationResult, Value};
use std::iter::Map;

mod io;
pub(crate) mod maintainer;
pub(crate) mod repl;

pub(crate) fn check_value_for_errors(wrapped_result: EvaluationResult) -> Result<Value, Error> {
    match (wrapped_result.value, wrapped_result.errors.as_slice()) {
        (Some(result), _) => Ok(result),
        (None, errors @ [_, ..]) => {
            let serialized_errors: Vec<String> =
                Map::collect(errors.iter().map(|error| error.fancy_format_str()));
            let mono_error = format!("```\n{}\n```", serialized_errors.join("\n"));
            Err(Error::from(mono_error))
        }
        (None, []) => Err(Error::from(
            "There was no result nor error! This shouldn't really happen.",
        )),
    }
}

pub(crate) fn add_embed_field(
    mut embed: CreateEmbed,
    name: &str,
    value: Option<&Value>,
) -> CreateEmbed {
    if let Some(value) = value {
        embed = embed.field(
            name,
            maintainer::format_field_value(format!("{}", value)),
            false,
        );
    }
    embed
}
