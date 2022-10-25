use serenity::{
    prelude::*,
    model::prelude::*,
    framework::StandardFramework, Client,
    framework::standard::{macros::{command,group}, CommandResult},
    utils::MessageBuilder
};

use serenity::async_trait;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
async fn rules(ctx: &Context, msg: &Message) -> CommandResult {
    let user = ctx.http.get_current_user().await?;

    let response = MessageBuilder::new()
        .push("Отправляю тебе свод правил, ")
        .push_bold_safe(&msg.author.name)
        .push("!")
        .build();
    msg.channel_id.say(&ctx.http, response).await?;

    msg.author.direct_message(&ctx.http, |m| {
        m.content("Ну ка улыбнись! Личка работает. Бот будет отправлять правила личным сообщением и уточнять ник здесь =)")
    }).await?;

    Ok(())
}

#[group]
#[commands(ping,rules)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // fn message(&self, ctx: Context, msg: Message) {
        // unimplemented!();
    // }
    
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let response = MessageBuilder::new()
            .push("Ничоси, ")
            .push_bold_safe(new_member.display_name())
            .push_line(" присоединился! О.О")
            .push_line("Ну здарова, чо!")
            .build();
        let defalt_channel = new_member.default_channel(&ctx.cache).expect("Couldn't retrieve the default channel!");
        let msg = defalt_channel.say(&ctx.http, response);
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let token = std::env::var("QUEENSCORSAR_TOKEN").expect("Couldn't fetch the API token from the environment");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);
    
    let intents = GatewayIntents::non_privileged() |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MEMBERS;
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
