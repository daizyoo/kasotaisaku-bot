use serenity::all::{CreateAllowedMentions, CreateMessage, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use tracing::{error, info};

fn mention(id: UserId) -> String {
    format!("<@{}>", id)
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.content == "mention" {
            let id = message.author.id;
            let mention = mention(id);

            if let Ok(msg) = message
                .channel_id
                .send_message(
                    ctx.http(),
                    CreateMessage::new()
                        .allowed_mentions(CreateAllowedMentions::new().users([id]))
                        .content(mention),
                )
                .await
            {
                info!("{:#?}", msg)
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
