use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
    Id,
};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct LFGSession {
    pub(crate) guild: Id<GuildMarker>,
    pub(crate) channel: Id<ChannelMarker>,
    pub(crate) original_message: Id<MessageMarker>,
    pub(crate) reply_message: Option<Id<MessageMarker>>,
    pub(crate) author: Id<UserMarker>,
    pub(crate) initial_tag: Id<RoleMarker>,
    pub(crate) participants: Vec<Id<UserMarker>>,
    pub(crate) added_participants: Vec<Id<UserMarker>>,
    pub(crate) excluded_participants: Vec<Id<UserMarker>>,
    pub(crate) interested_participants: Vec<Id<UserMarker>>,
    pub(crate) initial_number: u8,
    pub(crate) required_number: u8,
    pub(crate) expiry: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ChairmanUser {
    pub(crate) id: Id<UserMarker>,
    pub(crate) main_link: Option<Uuid>,
    pub(crate) linked_uuids: Option<Vec<Uuid>>,
    pub(crate) created: DateTime<Utc>,
    pub(crate) updated: DateTime<Utc>,
    pub(crate) administrator: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ChairmanLink {
    pub(crate) uuid: Uuid,
    pub(crate) parent: Id<UserMarker>,
    pub(crate) created: DateTime<Utc>,
    pub(crate) updated: DateTime<Utc>,
    pub(crate) last_username: String,
}
