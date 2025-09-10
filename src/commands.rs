use crate::Error;
use crate::nixpkgs::{NixpkgsPath, NixpkgsRepo};
use bytes::Bytes;
use openapi_github::apis::configuration::Configuration;
use openapi_github::apis::users_api::users_slash_get_by_username;
use openapi_github::models::UsersGetAuthenticated200Response;
use poise::serenity_prelude::{Color, CreateEmbed, Message};
use poise::{Context, CreateReply, command};
use snix_eval::{EvalIO, EvaluationResult, FileType, Value};
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[command(slash_command)]
pub(crate) async fn ping(ctx: Context<'_, (), Error>) -> Result<(), Error> {
    ctx.say("Mraowww!").await?;
    Ok(())
}

struct NixpkgsIo;

impl NixpkgsIo {
    fn ensure_inside(path: &Path) -> io::Result<PathBuf> {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            NixpkgsPath.join(path)
        };
        let canon = abs.canonicalize()?;
        if !canon.starts_with(&*NixpkgsPath) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Path {:?} is outside nixpkgs root", canon),
            ));
        }
        Ok(canon)
    }
}

impl EvalIO for NixpkgsIo {
    fn path_exists(&self, path: &Path) -> io::Result<bool> {
        let path = Self::ensure_inside(path)?;
        Ok(path.exists())
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read>> {
        let path = Self::ensure_inside(path)?;
        Ok(Box::new(fs::File::open(path)?))
    }

    fn file_type(&self, path: &Path) -> io::Result<FileType> {
        let path = Self::ensure_inside(path)?;
        let meta = fs::metadata(path)?;
        if meta.is_file() {
            Ok(FileType::Regular)
        } else if meta.is_dir() {
            Ok(FileType::Directory)
        } else {
            Ok(FileType::Symlink)
        }
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<(Bytes, FileType)>> {
        let path = Self::ensure_inside(path)?;
        let mut out = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let ftype = entry.file_type()?;
            let kind = if ftype.is_file() {
                FileType::Regular
            } else if ftype.is_dir() {
                FileType::Directory
            } else {
                FileType::Symlink
            };
            out.push((Bytes::from(name.as_bytes().to_owned()), kind));
        }
        Ok(out)
    }

    fn import_path(&self, path: &Path) -> io::Result<PathBuf> {
        Self::ensure_inside(path)
    }

    fn get_env(&self, key: &OsStr) -> Option<OsString> {
        std::env::var_os(key)
    }
}
#[command(slash_command)]
pub(crate) async fn eval(
    ctx: Context<'_, (), Error>,
    #[description = "Expression"] expression: String,
) -> Result<(), Error> {
    eval_discord_expression(ctx, expression).await?
}

#[command(context_menu_command = "Evaluate Nix code block", slash_command)]
pub(crate) async fn eval_code_block(
    ctx: Context<'_, (), Error>,
    #[description = "Message"] message: Message,
) -> Result<(), Error> {
    // Find the start of the Nix code block (`nix\n`).
    let expression = message
        .content
        .split_once("```nix\n")
        .ok_or("Couldn't find Nix code block!")?
        .1;

    // Find the end of the code block (`\n````).
    let expression = expression
        .rsplit_once("```")
        .ok_or("Couldn't find the end of the Nix code block!")?
        .0;

    // Call the original `eval` function with the extracted expression.
    eval_discord_expression(ctx, expression.to_string()).await?
}

async fn eval_discord_expression(
    ctx: Context<'_, (), Error>,
    expression: String,
) -> Result<Result<(), Error>, Error> {
    let response: String = {
        let mode = snix_eval::EvalMode::Strict;
        let builder = snix_eval::Evaluation::builder_pure()
            .mode(mode)
            .enable_import()
            .enable_impure(Some(Box::new(NixpkgsIo)));
        let evaluation = builder.build();
        let result: EvaluationResult = snix_eval::Evaluation::evaluate(
            evaluation,
            expression,
            Some(NixpkgsPath.as_path().into()),
        );
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
    Ok(Ok(()))
}

#[command(slash_command)]
pub(crate) async fn maintainer(
    ctx: poise::Context<'_, (), Error>,
    #[description = "Maintainer Name/Handle"] name: String,
) -> Result<(), Error> {
    let nixpkgs_repo = NixpkgsRepo
        .try_lock()
        .map_err(|_| "The nixpkgs repo is currently in use elsewhere. Try again later.")?;
    // We're gonna use this later for the embed image
    let mut github_username: Option<String> = None;
    let mut embed = nixpkgs_repo
        .as_ref()
        .map(|nixpkgs| {
            let nixpkgs_root = nixpkgs.path().parent().unwrap();
            let maintainer_expression =
                format!("(import ./maintainers/maintainer-list.nix).{}", name);
            let mode = snix_eval::EvalMode::Strict;
            let builder = snix_eval::Evaluation::builder_pure()
                .mode(mode)
                .enable_impure(Some(Box::new(NixpkgsIo)));
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

            // Set the github username for later use
            if let Some(username_value) = maintainer.select("github") {
                let quoted_username = format!("{}", username_value);
                let username = &quoted_username[1..&quoted_username.len() - 1];
                github_username = Some(String::from(username));
            };

            let mut embed: CreateEmbed = CreateEmbed::new()
                .title("Maintainer Info")
                .color(Color::from((35, 127, 235)));
            embed = add_embed_field(embed, "Name", maintainer.select("name"));
            embed = add_embed_field(embed, "Email", maintainer.select("email"));
            embed = add_embed_field(embed, "GitHub Username", maintainer.select("github"));
            embed = add_embed_field(embed, "GitHub ID", maintainer.select("githubId"));
            embed = add_embed_field(embed, "Matrix", maintainer.select("matrix"));

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

#[command(slash_command, owners_only)]
pub(crate) async fn nixpkgs_pull(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    // This can be expensive, so defer the interaction.
    ctx.defer().await?;
    let guard = NixpkgsRepo.lock().await;

    guard
        .as_ref()
        .map(|repo| {
            // fetch only tip
            let mut remote = repo
                .find_remote("origin")
                .or(Err("Could not find the upstream remote!"))?;
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.depth(1);
            remote
                .fetch(&["master"], Some(&mut fetch_options), None)
                .or(Err("The master branch is gone yo."))?;

            let fetch_head = repo
                .find_reference("FETCH_HEAD")
                .or(Err("The commit I just fetched fucking *vanished*"))?;
            let target = fetch_head
                .target()
                .ok_or_else(|| Error::from("No FETCH_HEAD target"))
                .or(Err("Couldn't target the fetched head."))?;

            // hard reset local branch to it
            let mut reference = repo.find_reference("refs/heads/master").or(Err(
                "Can't find the local master branch I plan to apply the work to!",
            ))?;
            reference
                .set_target(target, "Reset to latest upstream")
                .or(Err("Set target operation on the reflog shit itself..."))?;
            repo.set_head("refs/heads/master")
                .or(Err("Setting the new head shat itself."))?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .or(Err("Moving the local checkout failed."))?;

            Ok::<(), String>(())
        })
        .ok_or("Nixpkgs repo is not available!")??;

    ctx.say("Nixpkgs updated to upstream tip.").await?;
    Ok(())
}
