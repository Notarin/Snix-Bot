use crate::Error;
use crate::nixpkgs::NixpkgsRepo;
use poise::{Context, command};

pub(crate) mod snix;

#[command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn ping(ctx: Context<'_, (), Error>) -> Result<(), Error> {
    ctx.say("Mraowww!").await?;
    Ok(())
}

#[command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    owners_only
)]
pub(crate) async fn nixpkgs_pull(ctx: Context<'_, (), Error>) -> Result<(), Error> {
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

#[command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn noogle(ctx: Context<'_, (), Error>, function: String) -> Result<(), Error> {
    let function = function.trim().replace(" ", "").replace(".", "/");
    let url = format!("https://noogle.dev/f/{}", function);

    let resp = reqwest::get(&url).await.map_err(|_| "Error!".to_string())?;
    if resp.status().is_success() {
        ctx.say(&url).await?;
        Ok(())
    } else if resp.status().as_u16() == 404 {
        Err(Error::from("Function doesn't exist on Noogle!"))
    } else {
        panic!("Unexpected response!")
    }
}
