use crate::Error;
use poise::command;
use snix_eval::Value;

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
    let response = {
        let evaluation = snix_eval::Evaluation::builder_pure().build();
        let result = snix_eval::Evaluation::evaluate(evaluation, expression, None);

        match result.value {
            None => "Null!".to_string(),
            Some(Value::Null) => "Null!".to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Integer(i)) => i.to_string(),
            Some(Value::Float(f)) => f.to_string(),
            Some(Value::String(s)) => s.to_string(),
            _ => "<unhandled type>".to_string(),
        }
        // result is dropped here
    };

    ctx.say(response).await?;
    Ok(())
}
