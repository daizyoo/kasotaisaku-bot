use std::env;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::time::Duration;

use serenity::all::{CreateAllowedMentions, CreateMessage, GuildId, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use tracing::{error, info};

const SERVER_ID: GuildId = GuildId::new(1288316379069546527);
const TIME_OUT: Duration = Duration::from_secs(10);

struct ServerMembers;

impl TypeMapKey for ServerMembers {
    type Value = Vec<UserId>;
}

#[derive(Clone)]
struct Senders {
    s: Arc<Sender<bool>>,
}

struct MessageSenderContent;

impl TypeMapKey for MessageSenderContent {
    type Value = Senders;
}

fn mention(id: UserId) -> String {
    format!("<@{}>", id)
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        let mut sender_content = ctx.data.write().await;
        let sender = sender_content.get_mut::<MessageSenderContent>().unwrap();

        let e = sender.s.send(true);
        info!("{:?}", e);

        let (new_sender, receiver) = channel::<bool>();

        sender.s = Arc::new(new_sender);

        // let data = ctx.data.read().await;
        // if let Some(members) = data.get::<ServerMembers>() {
        //     info!("{:#?}", members)
        // }

        let http = ctx.http;
        tokio::spawn(async move {
            // let member = members.choose(&mut thread_rng()).unwrap();

            info!("tokio new thread");
            if let Err(_) = receiver.recv_timeout(TIME_OUT) {
                let id = message.author.id;
                let mention = mention(id);

                if let Ok(msg) = message
                    .channel_id
                    .send_message(
                        http,
                        CreateMessage::new()
                            .allowed_mentions(CreateAllowedMentions::new().users([id]))
                            .content(mention),
                    )
                    .await
                {
                    let m = msg.content;
                    info!("{:#?}", m)
                }
            } else {
                info!("recv message")
            }
        });
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let members = SERVER_ID.members(ctx.http(), None, None).await.unwrap();
        let members = members.iter().map(|x| x.user.id).collect();

        let mut data = ctx.data.write().await;
        let data = data.get_mut::<ServerMembers>().unwrap();

        println!("{:#?}", members);

        *data = members;
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
        .type_map_insert::<ServerMembers>(Vec::new())
        .await
        .expect("Err creating client");
    {
        let (s, _) = channel::<bool>();
        let mut data = client.data.write().await;
        data.insert::<MessageSenderContent>(Senders { s: Arc::new(s) });
    }

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
