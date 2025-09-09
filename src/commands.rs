use crate::Error;
use crate::nixpkgs::NixpkgsRepo;
use poise::serenity_prelude::{Color, CreateEmbed};
use poise::{CreateReply, command};
use snix_eval::{EvaluationResult, Value};
use std::path::PathBuf;

#[command(slash_command)]
pub(crate) async fn ping(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    ctx.say("Mraowww!").await?;
    Ok(())
}

#[command(slash_command)]
pub(crate) async fn eval(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Expression"] expression: String,
) -> Result<(), Error> {
    let response: String = {
        let mode = snix_eval::EvalMode::Strict;
        let builder = snix_eval::Evaluation::builder_pure().mode(mode);
        let evaluation = builder.build();
        let result: EvaluationResult =
            snix_eval::Evaluation::evaluate(evaluation, expression, None);
        format!(
            "{}",
            result
                .value
                .ok_or("There was an error in the nix evaluation.")?
        )
    };

    let fmt_config = alejandra::config::Config::default();
    let formatted = alejandra::format::in_memory(String::from(""), response, fmt_config).1;

    let code_block_response: String = format!("```nix\n{}\n```", formatted);
    ctx.say(code_block_response).await?;
    Ok(())
}

#[command(slash_command)]
pub(crate) async fn maintainer(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Maintainer Name/Handle"] name: String,
) -> Result<(), Error> {
    let nixpkgs_repo = NixpkgsRepo
        .try_lock()
        .map_err(|_| "The nixpkgs repo is currently in use elsewhere. Try again later.")?;
    let reply: CreateReply = nixpkgs_repo
        .as_ref()
        .map(|nixpkgs| {
            let nixpkgs_root = nixpkgs.path().parent().unwrap();
            let maintainer_expression =
                format!("(import ./maintainers/maintainer-list.nix).{}", name);
            let mode = snix_eval::EvalMode::Strict;
            let builder = snix_eval::Evaluation::builder_impure().mode(mode);
            let evaluation = builder.build();
            let result: EvaluationResult = snix_eval::Evaluation::evaluate(
                evaluation,
                maintainer_expression,
                Some(PathBuf::from(nixpkgs_root)),
            );
            let maintainer = result
                .value
                .unwrap()
                .to_attrs()
                .map_err(|_| "Expression wasn't an attrset!")?;

            let mut embed: CreateEmbed = CreateEmbed::new()
                .title("Maintainer Info")
                .color(Color::from((35, 127, 235)));
            embed = add_embed_field(embed, "Name", maintainer.select("name"));
            embed = add_embed_field(embed, "Email", maintainer.select("email"));
            embed = add_embed_field(embed, "GitHub Username", maintainer.select("github"));
            embed = add_embed_field(embed, "GitHub ID", maintainer.select("githubId"));
            embed = add_embed_field(embed, "Matrix", maintainer.select("matrix"));

            Ok::<CreateReply, Error>(CreateReply::default().embed(embed).reply(true))
        })
        .transpose()?
        .ok_or("The nixpkgs repo has not been set up. Try again later.")?;

    ctx.send(reply).await?;
    Ok(())
}

fn add_embed_field(mut embed: CreateEmbed, name: &str, value: Option<&Value>) -> CreateEmbed {
    if let Some(value) = value {
        embed = embed.field(name, format_field_value(format!("{}", value)), false);
    }
    embed
}

fn format_field_value(string: String) -> String {
    let mut inner: &str = &string;

    if inner.starts_with('"') {
        inner = &inner[1..];
    }
    if inner.ends_with('"') {
        inner = &inner[..inner.len() - 1];
    }

    format!("`{}`", inner)
}
