use crate::Error;
use poise::serenity_prelude::Message;
use poise::{Context, command};

#[command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn eval(
    ctx: Context<'_, (), Error>,
    #[description = "Expression"] expression: String,
) -> Result<(), Error> {
    eval_discord_expression(ctx, expression).await?
}

#[command(
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    context_menu_command = "Evaluate Nix code block"
)]
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
        use crate::commands::snix;
        use crate::commands::snix::io::NixpkgsIo;
        use crate::nixpkgs::NixpkgsPath;
        use snix_eval::{Evaluation, EvaluationResult};
        use tokio::time::{Duration, timeout};

        let eval_timeout: Duration = Duration::from_secs(2);
        let output: Result<String, Error> = timeout(
            eval_timeout,
            tokio::task::spawn_blocking(move || {
                let mode = snix_eval::EvalMode::Strict;

                let fake_derivation_builder = Evaluation::evaluate(
                    snix_eval::Evaluation::builder_pure()
                        .mode(mode)
                        .enable_import()
                        .enable_impure(Some(Box::new(NixpkgsIo)))
                        .build(),
                    "arg: arg // {out={type=null;outputName=null;};}",
                    Some(NixpkgsPath.as_path().into()),
                )
                .value
                .unwrap();

                let fake_placeholder = Evaluation::evaluate(
                    snix_eval::Evaluation::builder_pure()
                        .mode(mode)
                        .enable_import()
                        .enable_impure(Some(Box::new(NixpkgsIo)))
                        .build(),
                    "arg: arg",
                    Some(NixpkgsPath.as_path().into()),
                )
                .value
                .unwrap();

                let builder = snix_eval::Evaluation::builder_pure()
                    .mode(mode)
                    .enable_import()
                    .add_builtins(vec![
                        ("derivation", fake_derivation_builder),
                        ("placeholder", fake_placeholder),
                    ])
                    .enable_impure(Some(Box::new(NixpkgsIo)));

                let evaluation = builder.build();
                let result: EvaluationResult = Evaluation::evaluate(
                    evaluation,
                    expression,
                    Some(NixpkgsPath.as_path().into()),
                );

                Ok(format!("{}", snix::check_value_for_errors(result)?))
            }),
        )
        .await
        .map_err(|_| {
            format!(
                "Evaluation took too long. Max eval time is {} seconds.",
                eval_timeout.as_secs()
            )
        })?
        .unwrap();
        output?
    };

    let fmt_config = alejandra::config::Config::default();
    let formatted = alejandra::format::in_memory(String::from(""), response, fmt_config).1;

    let code_block_response: String = format!("```nix\n{}\n```", formatted);
    ctx.say(code_block_response).await?;
    Ok(Ok(()))
}
