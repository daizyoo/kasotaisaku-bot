use std::env;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use serenity::all::{CreateMessage, GuildId, UserId};
use serenity::async_trait;
use serenity::http::Http;
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

struct MessageSenderContent;

impl TypeMapKey for MessageSenderContent {
    type Value = Senders;
}

struct Senders(Sender<bool>);

impl Senders {
    fn send(&self) {
        let r = self.0.send(true);
        info!("{:?}", r);
    }
    fn update(&mut self, new_sender: Sender<bool>) {
        self.0 = new_sender
    }
}

fn mention(id: UserId) -> String {
    format!("<@{}>", id)
}

fn timer(http: Arc<Http>, msg: Message, receiver: Receiver<bool>, user_id: UserId) {
    tokio::spawn(async move {
        // let member = members.choose(&mut thread_rng()).unwrap();
        info!("tokio new thread");
        if let Err(_) = receiver.recv_timeout(TIME_OUT) {
            let mention = mention(user_id);
            let create_msg = CreateMessage::new().content(mention);

            if let Ok(msg) = msg.channel_id.send_message(http, create_msg).await {
                info!("{:#?}", msg.content)
            }
        } else {
            info!("recv message")
        }
    });
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut sender_content = ctx.data.write().await;
        let sender = sender_content.get_mut::<MessageSenderContent>().unwrap();

        sender.send();

        let (new_sender, receiver) = channel::<bool>();

        sender.update(new_sender);

        // let data = ctx.data.read().await;
        // if let Some(members) = data.get::<ServerMembers>() {
        //     info!("{:#?}", members)
        // }

        let user_id = msg.author.id;
        timer(ctx.http, msg, receiver, user_id);
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
        data.insert::<MessageSenderContent>(Senders(s));
    }

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
