use serenity::{
    builder::{CreateEmbed, CreateMessage}, 
    client::Context, 
    model::channel::Message, 
    framework::standard::CommandResult
};

pub fn create_embed(title: &str, description: &str, color: u32) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
}

pub async fn send_embed(ctx: &Context, msg: &Message, title: &str, description: &str, color: u32) -> CommandResult {
    let embed = create_embed(title, description, color);
    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;
    Ok(())
}
