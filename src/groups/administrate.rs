extern crate serenity;

use serenity::prelude::*;
use serenity::model::{
    prelude::*,
};

use serenity::framework::standard::{
    Args, Delimiter, CommandResult,
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

#[command("channel")]
#[sub_commands(add_channel, edit_channel, delete_channel)]
pub async fn channel_related(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command("add")]
pub async fn add_channel(ctx: &Context, msg: &Message, real_args: Args) -> CommandResult {
    let mut args = Args::new(real_args.message(), &[Delimiter::Single(' ')]);

    let name = args.single::<String>()?;

    let _ = msg
    .channel_id
    .say(&ctx.http, name.clone())
    .await;

    let kkind = args.single::<String>()?;

    let kind = match kkind.to_lowercase().as_str() {
        "text" => ChannelType::Text,
        "voice" => ChannelType::Voice,
        _ => ChannelType::Text,
    };
    let topic = args.single::<String>()?;
    let category = args.single::<u64>()?;
    let pos = args.single::<u32>()?;

    let _ = msg
    .channel_id
    .say(&ctx.http, &format!("{}\n{}\n{}\n{}\n{}", name, kkind, topic, category, pos))
    .await;

    let _ = &msg.guild_id.unwrap()
    .create_channel(&ctx.http,
        |c|
        c
        .name(name)
        .kind(kind)
        .topic(topic)
        .category(category)
        .position(pos - 1)
    )
    .await;

    Ok(())
}

#[command("edit")]
pub async fn edit_channel(ctx: &Context, msg: &Message) -> CommandResult {
    unimplemented!();
}

#[command("del")]
pub async fn delete_channel(ctx: &Context, msg: &Message, real_args: Args) -> CommandResult {
    let mut args = Args::new(real_args.message(), &[Delimiter::Single(' ')]);

    let id = args.single::<u64>()?;
    let _ = ChannelId(id).delete(&ctx.http).await;

    Ok(())
}

#[command]
pub async fn role_related(ctx: &Context, msg: &Message) -> CommandResult {
    unimplemented!();
}


