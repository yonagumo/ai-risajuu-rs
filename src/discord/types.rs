#![allow(dead_code)]

use serenity::{
    model::{
        channel::Message,
        id::{ChannelId, GuildId, UserId},
    },
    prelude::*,
};

#[derive(Debug, Clone)]
pub struct DiscordEvent {
    pub ctx: Context,
    pub msg: Message,
    pub place: PlaceIdentifier,
}

#[derive(Debug, Clone)]
pub enum PlaceIdentifier {
    DM(ChannelId, UserId),
    Server(GuildId, Option<ChannelId>, ChannelId),
}

impl PlaceIdentifier {
    pub async fn new(ctx: &Context, msg: &Message) -> PlaceIdentifier {
        if let Some(guild_id) = msg.guild_id {
            let category = msg.category_id(&ctx.http).await;
            Self::Server(guild_id, category, msg.channel_id)
        } else {
            PlaceIdentifier::DM(msg.channel_id, msg.author.id)
        }
    }

    pub fn is_dm(&self) -> bool {
        match self {
            Self::DM(_, _) => true,
            _ => false,
        }
    }

    pub fn is_server(&self) -> bool {
        match self {
            Self::Server(_, _, _) => true,
            _ => false,
        }
    }
}
