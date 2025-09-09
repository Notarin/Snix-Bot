use crate::Error;
use crate::nixpkgs::NixpkgsRepo;
use log::debug;
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
        format!("{}", result.value.unwrap())
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
    #[description = "Maintainer Username"] uname: String,
) -> Result<(), Error> {
    match NixpkgsRepo.try_lock() {
        Ok(nixpkgs_repo) => {
            let response: String = {
                let nixpkgs_root = nixpkgs_repo.path().parent().unwrap();
                let maintainer_expression =
                    format!("(import ./maintainers/maintainer-list.nix).{}", uname);
                debug!(
                    "Got a maintainer request. Evaluating the following: {}",
                    maintainer_expression
                );

                let mode = snix_eval::EvalMode::Lazy;
                let builder = snix_eval::Evaluation::builder_impure()
                    .mode(mode)
                    .enable_import();
                let evaluation = builder.build();
                let result: EvaluationResult = snix_eval::Evaluation::evaluate(
                    evaluation,
                    maintainer_expression,
                    Some(PathBuf::from(nixpkgs_root)),
                );

                format!("{}", result.value.unwrap())
            };
            ctx.say(response).await?;
        }
        Err(_) => {
            ctx.say("For whatever reason, nixpkgs is not ready. Try again later.")
                .await?;
        }
    }
    Ok(())
}
