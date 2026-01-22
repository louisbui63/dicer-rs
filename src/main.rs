use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;

mod evaluator;
mod parser;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, mut msg: Message) {
        let inter = if msg.content.ends_with("is this true?") {
            "!dice $1d6".to_owned()
        } else {
            msg.content.clone()
        };

        let mut m = inter.clone();
        println!("{m}");
        m.truncate(6);

        if inter == "!dice help" || msg.content == "!dicer help" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Dicer is a dice roller bot designed for tabletop rpg. It is based on an innovative representation of rolls as mathematical expressions, allowing endless possibilities, end thus making it suitable no matter the rules you are using.
!dice followed by a command outputs the result of this command.
there also some specific commands :
!dice help		: displays this help
!dice clear		: clears the channel").await {
                println!("error sending message : {:?}", why);
            }
        } else if m == "!dice " {
            let mut content = inter[6..].to_owned();
            if !content.ends_with('\n') {
                content.push('\n');
            }
            println!("{:?}", content);
            let to_unwrap = crate::parser::tokenize(content);

            if let Err(e) = to_unwrap.clone() {
                if let Err(why) = msg.channel_id.say(&ctx.http, e).await {
                    eprintln!("error sending message : {:?}", why);
                    return;
                }
            }
            let tokens = to_unwrap.unwrap();
            println!("{:?}", tokens);

            let mut i = 0;
            let to_unwrap = crate::parser::parse(&tokens, &mut i);
            if let Err(e) = to_unwrap.clone() {
                if let Err(why) = msg.channel_id.say(&ctx.http, e).await {
                    eprintln!("error sending message : {:?}", why);
                    return;
                }
            }
            let parsed = to_unwrap.unwrap();
            println!("{:?}", parsed);

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
            println!("{}", evaluated);

            if evaluated != "" {
                if let Err(why) = msg.channel_id.say(&ctx.http, evaluated).await {
                    eprintln!("error sending message : {:?}", why);
                }
            }
        }
    }
}

#[cfg(debug_assertions)]
fn main() {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer).unwrap();
        let to_unwrap = crate::parser::tokenize(buffer.to_owned());

        println!("{}", buffer);

        if let Err(e) = to_unwrap.clone() {
            eprintln!("{}", e);
            continue;
        }
        let tokens = to_unwrap.unwrap();
        println!("{:?}", tokens);

        let mut i = 0;
        let to_unwrap = crate::parser::parse(&tokens, &mut i);
        if let Err(e) = to_unwrap.clone() {
            eprintln!("{}", e);
            continue;
        }
        let parsed = to_unwrap.unwrap();
        println!("{:?}", parsed);

        let to_unwrap = crate::evaluator::evaluate(&parsed, &mut std::collections::HashMap::new());
        if let Err(e) = to_unwrap.clone() {
            eprintln!("{}", e);
            continue;
        }
        let evaluated = to_unwrap.unwrap();

        println!("{}", evaluated);
    }
}

#[cfg(not(debug_assertions))]
#[tokio::main]
async fn main() {
    let token = std::fs::read_to_string("token.txt")
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
