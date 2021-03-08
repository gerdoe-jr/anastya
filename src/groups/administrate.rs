extern crate serenity;

use serenity::prelude::*;
use serenity::model::{
    prelude::*,
};

use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

#[command]
async fn server_related(ctx: &Context, msg: &Message) -> CommandResult {
    unimplemented!();
}

#[command]
pub async fn category_related(ctx: &Context, msg: &Message) -> CommandResult {
    unimplemented!();
}

#[command]
pub async fn channel_related(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single::<String>().unwrap().to_lowercase().as_str() {
        "add" => {
            let _ = &msg.guild_id.unwrap()
            .create_channel(&ctx.http,
                |c|
                c
                .name(args.single::<String>().unwrap())
                .kind(match args.single::<String>().unwrap().to_lowercase().as_str() {
                    "text" => ChannelType::Text,
                    "voice" => ChannelType::Voice,
                    _ => ChannelType::Text,
                })
                .topic(args.single::<String>().unwrap())
                .category(args.single::<u64>().unwrap())
                .position(args.single::<u32>().unwrap()))
                
            .await;
        },
        "del" => (),
        "edit" => (),
        _ => (),
    }

    Ok(())
}

#[command]
pub async fn role_related(ctx: &Context, msg: &Message) -> CommandResult {
    unimplemented!();
}


