use serenity::{
    prelude::*,
    model::prelude::*,
    framework::StandardFramework, Client,
    framework::standard::{macros::{command,group}, CommandResult}
};

use serenity::async_trait;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // fn message(&self, ctx: Context, msg: Message) {
        // unimplemented!();
    // }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let token = std::env::var("QUEENSCORSAR_TOKEN").expect("Couldn't fetch the API token from the environment");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);
    
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
    Ok(())
}
