use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;

use std::fs;

mod evaluator;
mod parser;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut m = msg.content.clone();
        m.truncate(6);

        if msg.content == "!dice help" || msg.content == "!dicer help" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Dicer is a dice roller bot designed for tabletop rpg. It is based on an innovative representation of rolls as mathematical expressions, allowing endless possibilities, end thus making it suitable no matter the rules you are using.
!dice followed by a command outputs the result of this command.
there also some specific commands :
!dice help		: displays this help
!dice clear		: clears the channel").await {
                println!("error sending message : {:?}", why);
            }
        } else if m == "!dice " {
            let content = &msg.content[6..];
            let to_unwrap = crate::parser::tokenize(content.to_owned());

            if let Err(e) = to_unwrap.clone() {
                if let Err(why) = msg.channel_id.say(&ctx.http, e).await {
                    eprintln!("error sending message : {:?}", why);
                    return;
                }
            }
            let tokens = to_unwrap.unwrap();

            let mut i = 0;
            let to_unwrap = crate::parser::parse(&tokens, &mut i);
            if let Err(e) = to_unwrap.clone() {
                if let Err(why) = msg.channel_id.say(&ctx.http, e).await {
                    eprintln!("error sending message : {:?}", why);
                    return;
                }
            }
            let parsed = to_unwrap.unwrap();

            let to_unwrap =
                crate::evaluator::evaluate(&parsed, &mut std::collections::HashMap::new());
            if let Err(e) = to_unwrap.clone() {
                if e != "" {
                    if let Err(why) = msg.channel_id.say(&ctx.http, e).await {
                        eprintln!("error sending message : {:?}", why);
                        return;
                    }
                }
            }
            let evaluated = to_unwrap.unwrap();

            if evaluated != "" {
                if let Err(why) = msg.channel_id.say(&ctx.http, evaluated).await {
                    eprintln!("error sending message : {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = fs::read_to_string("token.txt")
        .expect("unable to find token, please check token.txt location");

    use serenity::model::gateway::GatewayIntents;
    let intents = GatewayIntents::all();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        println!("client error : {:?}", why);
    }
}
