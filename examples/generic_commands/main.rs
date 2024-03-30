//! If you need it, poise-annotated command functions can also be generic over the user data type
//! or error type
//!
//! The original use case for this feature was to have the same command in two different bots

#[poise::command(slash_command)]
pub async fn example<U: Sync, E>(ctx: poise::Context<'_, U, E>) -> Result<(), E> {
    ctx.say(format!(
        "My user data type is {} and the error type is {}",
        std::any::type_name::<U>(),
        std::any::type_name::<E>()
    ))
    .await
    .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() {
    let _example1 = example::<(), ()>();
    let _example2 = example::<String, Box<dyn std::error::Error>>();
}

/// Description of the command here
///
/// Here you can explain how the command \
/// is used and how it works.
#[poise::command(prefix_command, /* add more optional command settings here, like slash_command */)]
async fn cv(
    ctx: Context<'_>,
    #[description = "Description of arg1 here"] arg1: serenity::Member,
    #[description = "Description of arg2 here"] arg2: Option<u32>,
) -> Result<(), Error> {
    // Command code here

    Ok(())
}
