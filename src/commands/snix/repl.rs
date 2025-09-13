use crate::Error;
use crate::commands::snix;
use crate::commands::snix::io::NixpkgsIo;
use crate::nixpkgs::NixpkgsPath;
use poise::futures_util::future::join_all;
use poise::serenity_prelude::Message;
use poise::{Context, command};
use regex::{Captures, Regex};
use snix_eval::{Evaluation, EvaluationResult};
use tokio::time::{Duration, timeout};

#[command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn eval(
    ctx: Context<'_, (), Error>,
    #[description = "Expression"] expression: String,
) -> Result<(), Error> {
    eval_discord_expression(ctx, expression).await
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

    // Call the original `eval` function with the extracted Expression.
    eval_discord_expression(ctx, expression.to_string()).await
}

struct Assignment {
    name: String,
    value: String,
}

enum ToEvaluateType {
    Expression(String),
    Assignment(Vec<Assignment>),
}

async fn eval_discord_expression(
    ctx: Context<'_, (), Error>,
    to_evaluate: String,
) -> Result<(), Error> {
    let response: String = {
        let regex = Regex::new(r"(?m)^(?P<assignment>(?P<variable_name>[^\s]+)\s*= )").unwrap();
        let matches: Vec<Captures> = regex.captures_iter(&to_evaluate).collect();

        let to_eval_wrapped: ToEvaluateType = match matches.as_slice() {
            [] => ToEvaluateType::Expression(to_evaluate.to_string()),
            assignment_captures => {
                let mut assignments = Vec::new();
                for (index, capture) in assignment_captures.iter().enumerate() {
                    let assignment = capture.name("assignment").unwrap();
                    let expression_start_pos = assignment.end();
                    let end = if index + 1 < assignment_captures.len() {
                        assignment_captures[index + 1]
                            .name("assignment")
                            .unwrap()
                            .start()
                            - 2
                    } else {
                        to_evaluate.len() - 2
                    };
                    let variable_name = capture.name("variable_name").unwrap().as_str().to_string();
                    let expression = to_evaluate[expression_start_pos..end].trim().to_string();

                    assignments.push(Assignment {
                        name: variable_name,
                        value: expression,
                    });
                }
                ToEvaluateType::Assignment(assignments)
            }
        };

        match to_eval_wrapped {
            ToEvaluateType::Expression(expression) => {
                let output = evaluate_expression(expression).await?;
                let formatted = format(output);
                make_code_block(formatted)
            }
            ToEvaluateType::Assignment(assignments) => {
                let evaluated_list: Vec<(String, String)> =
                    join_all(assignments.into_iter().map(|assignment| async move {
                        let name = assignment.name;
                        let result = evaluate_expression(assignment.value).await?;
                        Ok::<_, Error>((name, result))
                    }))
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, Error>>()?;
                let evaluated_list: Vec<String> = evaluated_list
                    .into_iter()
                    .map(|entry| format!("  {} = {};", entry.0, entry.1))
                    .collect();
                let attr_set = format!("{{\n{}\n}}", evaluated_list.join("\n"));
                make_code_block(format(attr_set))
            }
        }
    };
    ctx.say(response).await?;
    Ok(())
}

fn make_code_block(string: String) -> String {
    let code_block_response: String = format!("```nix\n{}\n```", string);
    code_block_response
}

fn format(nix: String) -> String {
    let fmt_config = alejandra::config::Config::default();
    alejandra::format::in_memory(String::from(""), nix, fmt_config).1
}

async fn evaluate_expression(expression: String) -> Result<String, Error> {
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
            let result: EvaluationResult =
                Evaluation::evaluate(evaluation, expression, Some(NixpkgsPath.as_path().into()));

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
    output
}
