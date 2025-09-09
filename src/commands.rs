use crate::Error;
use crate::nixpkgs::NixpkgsRepo;
use poise::command;
use snix_eval::EvaluationResult;
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
    let result: Result<String, String> = nixpkgs_repo
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
            format!("{}", result.value.unwrap())
        })
        .ok_or_else(|| String::from("The nixpkgs repo has not been set up. Try again later."));
    ctx.say(result?).await?;
    Ok(())
}
