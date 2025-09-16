use crate::Error;
use crate::commands::snix;
use crate::commands::snix::io::NixpkgsIo;
use crate::nixpkgs::{NIXPKGS_PATH, NIXPKGS_REPO};
use openapi_github::apis::configuration::Configuration;
use openapi_github::apis::users_api::users_slash_get_by_username;
use openapi_github::models::UsersGetAuthenticated200Response;
use poise::serenity_prelude::{AutocompleteChoice, Color, CreateAutocompleteResponse, CreateEmbed};
use poise::{Context, CreateReply, command};
use snix_eval::{Evaluation, EvaluationResult, Value};
use std::path::PathBuf;

#[allow(clippy::unused_async)]
async fn autocomplete_maintainer(
    _ctx: Context<'_, (), Error>,
    partial: &str,
) -> CreateAutocompleteResponse {
    let mode = snix_eval::EvalMode::Strict;
    let builder = snix_eval::Evaluation::builder_pure()
        .mode(mode)
        .enable_import()
        .enable_impure(Some(Box::new(NixpkgsIo)));
    let evaluation = builder.build();
    let result: EvaluationResult = Evaluation::evaluate(
        evaluation,
        "(import ./lib).maintainers",
        Some(NIXPKGS_PATH.as_path().into()),
    );

    let mut maintainers: Vec<String> = match snix::check_value_for_errors(result) {
        Ok(Value::Attrs(attrs)) => attrs
            .keys()
            .map(|maintainer| {
                let mut maintainer: &str = &maintainer.to_string();

                if maintainer.starts_with('"') {
                    maintainer = &maintainer[1..];
                }
                if maintainer.ends_with('"') {
                    maintainer = &maintainer[..maintainer.len() - 1];
                }
                maintainer.to_string()
            })
            .collect(),
        _ => Vec::new(),
    };

    let maintainers: Vec<String> = maintainers
        .iter_mut()
        .filter(|maintainer| maintainer.starts_with(partial))
        .map(|name| (*name).to_string())
        .collect::<Vec<String>>();
    let choices = maintainers.iter().map(AutocompleteChoice::from).collect();

    let mut autocomplete_response = CreateAutocompleteResponse::new();
    autocomplete_response = autocomplete_response.set_choices(choices);
    autocomplete_response
}

#[command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn maintainer(
    ctx: Context<'_, (), Error>,
    #[autocomplete = "autocomplete_maintainer"]
    #[description = "Maintainer Name/Handle"]
    name: String,
) -> Result<(), Error> {
    let nixpkgs_repo = NIXPKGS_REPO
        .try_lock()
        .map_err(|_| "The nixpkgs repo is currently in use elsewhere. Try again later.")?;
    // We're gonna use this later for the embed image
    let mut github_username: Option<String> = None;
    let mut embed = nixpkgs_repo
        .as_ref()
        .map(|nixpkgs| {
            let nixpkgs_root = nixpkgs.path().parent().unwrap();
            let maintainer_expression =
                format!("(import ./maintainers/maintainer-list.nix).{name}");
            let mode = snix_eval::EvalMode::Strict;
            let builder = snix_eval::Evaluation::builder_pure()
                .mode(mode)
                .enable_impure(Some(Box::new(NixpkgsIo)));
            let evaluation = builder.build();
            let result: EvaluationResult = Evaluation::evaluate(
                evaluation,
                maintainer_expression,
                Some(PathBuf::from(nixpkgs_root)),
            );
            let maintainer = snix::check_value_for_errors(result)?
                .to_attrs()
                .map_err(|_| "Expression wasn't an attrset!")?;

            // Set the github username for later use
            if let Some(username_value) = maintainer.select("github") {
                let quoted_username = format!("{username_value}");
                let username = &quoted_username[1..&quoted_username.len() - 1];
                github_username = Some(String::from(username));
            }

            let mut embed: CreateEmbed = CreateEmbed::new()
                .title("Maintainer Info")
                .color(Color::from((35, 127, 235)));
            embed = snix::add_embed_field(embed, "Name", maintainer.select("name"));
            embed = snix::add_embed_field(embed, "Email", maintainer.select("email"));
            embed = snix::add_embed_field(embed, "GitHub Username", maintainer.select("github"));
            embed = snix::add_embed_field(embed, "GitHub ID", maintainer.select("githubId"));
            embed = snix::add_embed_field(embed, "Matrix", maintainer.select("matrix"));

            Ok::<CreateEmbed, Error>(embed)
        })
        .ok_or("The nixpkgs repo has not been set up. Try again later.")??;

    if let Some(username) = github_username {
        let maintainer_github_account =
            users_slash_get_by_username(&Configuration::default(), &username)
                .await
                .map_err(|_| "Couldn't fetch github user")?;
        let avatar: String = match maintainer_github_account {
            UsersGetAuthenticated200Response::PrivateUser(user) => user.avatar_url,
            UsersGetAuthenticated200Response::PublicUser(user) => user.avatar_url,
        };
        embed = embed.thumbnail(avatar);
    }

    ctx.send(CreateReply::default().embed(embed).reply(true))
        .await?;
    Ok(())
}

pub fn format_field_value(string: &str) -> String {
    let mut inner: &str = string;

    if inner.starts_with('"') {
        inner = &inner[1..];
    }
    if inner.ends_with('"') {
        inner = &inner[..inner.len() - 1];
    }

    format!("`{inner}`")
}
