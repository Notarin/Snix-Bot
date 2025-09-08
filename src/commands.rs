use crate::Error;
use crate::snix::convert_expression_to_string;
use poise::command;
use snix_eval::EvaluationResult;

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
        let evaluation = snix_eval::Evaluation::builder_pure().build();
        let result: EvaluationResult =
            snix_eval::Evaluation::evaluate(evaluation, expression, None);
        convert_expression_to_string(result).unwrap()
    };

    let formatted_response: String = format!("```\n{}\n```", response);
    ctx.say(formatted_response).await?;
    Ok(())
}
