use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;

use rand::prelude::*;

use std::fs;
use std::collections::HashMap;

fn is_operator(i: char) -> bool {
    return i == 'r' || i == 'd' || i == 'x' || i == 'k' || i == 'K' || i == 's'
}

fn get_precedence() -> HashMap<String, usize>{
    let mut out = HashMap::new();
    out.insert("r".to_owned(), 6);
    out.insert("d".to_owned(), 5);
    out.insert("x".to_owned(), 0);
    out.insert("k".to_owned(), 2);
    out.insert("K".to_owned(), 2);
    out.insert("s".to_owned(), 1);

    return out;
}

fn dop(t1: (String, String), t2: (String, String)) -> (String, String) {
    // println!("d\t{:?}\t{:?}", t1, t2);
    let nb = t1.1.parse::<isize>().unwrap();
    let max = t2.1.parse::<isize>().unwrap();

    if nb < 1 {return ("array".to_owned(), "".to_owned())}

    let mut result = vec!();
    
    let mut rng = thread_rng();

    if max < 0 {
        for _ in 1..=nb {
            result.push("0".to_owned());
        }
    }
    else {
        for _ in 1..=nb {
            result.push(rng.gen_range(1..=max).to_string());
        }
    }

    return ("array".to_owned(), result.join(","));
}

fn xop(t1: (String, String), t2: Vec<(String, String)> ) -> (String, String) {
    // DEBUG :
    // println!("x\t{:?}\t{:?}", t1, t2);
    
    let nb = t1.1.parse::<usize>().unwrap();
    let mut ret = "".to_owned();
    for i in 1..=nb {
        ret.push_str(&t2[i].1[..]);
        ret += ";";
    }
    ret.truncate(ret.len() -1);
    return ("biarray".to_owned(), ret);
}

fn rop(t1: (String, String), t2: (String, String)) -> (String, String) {
    
    // println!("r\t{:?}\t{:?}", t1, t2);
    let start = t1.1.parse::<isize>().unwrap();
    let end = t2.1.parse::<isize>().unwrap();
    let mut rng = thread_rng();
    return ("number".to_owned(), rng.gen_range(start..=end).to_string());
}

fn kop(t1: (String, String), t2: (String, String)) -> (String, String) {
    // println!("k\t{:?}\t{:?}", t1, t2);
    let vals: Vec<&str> = t1.1.split(",").collect();
    let qt = t2.1.parse::<usize>().unwrap();
    let mut ivals = vec!();
    for i in vals {
        let v = i.parse::<isize>().unwrap();
        ivals.push(v);
    }
    ivals.sort();
    ivals.truncate(qt);
    let mut ret = vec!();
    for i in ivals {ret.push(i.to_string())};
    return ("array".to_owned(), ret.join(","));
}

fn ukop(t1: (String, String), t2: (String, String)) -> (String, String) {
    // println!("K\t{:?}\t{:?}", t1, t2);
    let vals: Vec<&str> = t1.1.split(",").collect();
    let qt = t2.1.parse::<usize>().unwrap();
    let mut ivals = vec!();
    for i in vals {
        let v = i.parse::<isize>().unwrap();
        ivals.push(v);
    }
    ivals.sort();
    ivals.reverse();
    ivals.truncate(qt);
    let mut ret = vec!();
    for i in ivals {ret.push(i.to_string())};
    return ("array".to_owned(), ret.join(","));
}

fn eval(pos: usize, stack: Vec<(String, String)>) -> ((String, String), usize) {
    if stack[pos].0 == "array" || stack[pos].0 == "number" || stack[pos].0 == "biarray" {
        return (stack[pos].clone(), 0)
    }
    else {
        match &stack[pos].1[..] {
            "x" => {
                let mut op2: Vec<(String, String)> = vec!();
                let (_, i) = eval(pos-1, stack.clone());
                let (op1, j) = eval(pos-i-2, stack.clone());
                let qt = op1.1.parse::<isize>().unwrap();

                for _ in 0..=qt {
                    let (op, _) = eval(pos-1, stack.clone());
                    op2.push(op);
                }
                let a = xop(op1, op2);

                return (a, 2+i+j);
            }
            "d" => {
                let (op2, i) = eval(pos-1, stack.clone());
                let (op1, j) = eval(pos-i-2, stack.clone());
                let a = dop(op1, op2);
                return (a, 2+i+j);
            }
            "r" => {
                let (op2, i) = eval(pos-1, stack.clone());
                let (op1, j) = eval(pos-i-2, stack.clone());
                let a = rop(op1, op2);
                return (a, 2+i+j);
            }
            "k" => {
                let (op2, i) = eval(pos-1, stack.clone());
                let (op1, j) = eval(pos-i-2, stack.clone());
                let a = kop(op1, op2);
                return (a, 2+i+j);
            }
            "K" => {
                let (op2, i) = eval(pos-1, stack.clone());
                let (op1, j) = eval(pos-i-2, stack.clone());
                let a = ukop(op1, op2);
                return (a, 2+i+j);
            }
            "s" => {
                let (op, i) = eval(pos-1, stack.clone());
                let el: Vec<&str> = op.1.split(",").collect();
                let mut a = 0;
                for i in el {
                    let b = i.parse::<isize>().unwrap();
                    a += b;
                }
                return (("number".to_owned(), a.to_string()), i+1);
            }
            _ => {}
        }
    }


    todo!();
}

fn parse(c: String) -> Vec<String> {

    let mut tokens = vec!();
    let mut current_type = "";
    let mut buffer = "".to_owned();
    for i in c.chars() {
        if i.is_numeric() {
            if current_type != "number" && current_type != "" {
                tokens.push((current_type, buffer));
                buffer = "".to_owned();
            }
            current_type = "number";
            buffer.push(i);
        }
        else if is_operator(i) {
            if current_type != "" {
				tokens.push((current_type, buffer));
				buffer = "".to_owned();
			}
			current_type = "operator";
			buffer.push(i);
        }
        else if i == '(' {
            if current_type != "" {
                tokens.push((current_type, buffer));
                buffer = "".to_owned();
            }
            current_type = "lparen";
            buffer.push(i);
        }
        else if i == ')' {
            if current_type != "" {
                tokens.push((current_type, buffer));
                buffer = "".to_owned();
            }
            current_type = "rparen";
            buffer.push(i);
        }
    }
    tokens.push((current_type, buffer));

    let mut oustack: Vec<(String, String)> = vec!();
    let mut opstack: Vec<(String, String)> = vec!();

    let precedence = get_precedence();

    for i in tokens {
        match i.0 {
            "number" => {
                oustack.push((i.0.to_owned(), i.1));
            }
            "operator" => {
                while opstack.len() > 0 && (opstack[opstack.len()-1].0 == "operator" && (precedence[&opstack[opstack.len()-1].1] > precedence[&i.1] || precedence[&opstack[opstack.len()-1].1] /*should also be leftasso*/== precedence[&i.1])) {
					oustack.push(opstack[opstack.len()-1].clone());
					opstack.truncate(opstack.len() -1);
				}
				opstack.push((i.0.to_owned(), i.1));
            }
            "lparen" => {
                opstack.push((i.0.to_owned(), i.1));
            }
            "rparen" => {
                while opstack[opstack.len() -1].0 != "lparen" {
					oustack.push(opstack[opstack.len()-1].clone());
					opstack.truncate(opstack.len() -1);
				}
				opstack.truncate(opstack.len() -1);
            }

            _ => {}
        }

    }

    while opstack.len() != 0 {
        oustack.push(opstack[opstack.len()-1].clone());
		opstack.truncate(opstack.len() -1);
    }

    println!("{:?}", oustack);

    let (out, _) = eval(oustack.len() -1, oustack);

    println!("{:?}", out);

    let a: Vec<&str> = out.1.split(";").collect();
    let mut t: Vec<Vec<&str>> = vec!();

    for i in a {
        t.push(i.split(",").collect());
    }


    let mut out: Vec<String> = vec!();
    for i in t {
        out.push(i.join(","));
    }

    return out;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut m = msg.content.clone();
        m.truncate(6);

        if msg.content == "!dice help" || msg.content == "!dicer help" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Dicer is a dice roller bot designed for tabletop rpg. It is based on an innovative representation of rolls as mathematical expressions, allowing endless possibilities, end thus making it suitable no matter the rules you are using.
the following operators are available :
	- ndm rolls n m-sized dices and put the results in an array
	- nxm repeats n times m and put everything in a vertical array
	- nrm takes a random integer between n and m, both included
	- ns  sums the elements of the array n
	- nKm keeps only the m highest elements of n
	- nkm keeps only the m lowest elements of n
!dice followed by a command outputs the result of this command.
there also some specific commands :
!dice help		: displays this help
!dice clear		: clears the channel").await {
                println!("error sending message : {:?}", why);
            }
        }

        else if m == "!dice " {
            let content = &msg.content[6..];
            let out = parse(content.to_owned());
            for i in out {
                if let Err(why) = msg.channel_id.say(&ctx.http, i).await {
                    println!("error sending message : {:?}", why);
                }
            }
        }

    }
}

#[tokio::main]
async fn main() {
    let token = fs::read_to_string("token.txt").expect("unable to find token, please check token.txt location");
    let mut client = Client::builder(&token).event_handler(Handler).await.expect("error creating client");

    if let Err(why) = client.start().await {
        println!("client error : {:?}", why);
    }
}
