use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Latency;
use twilight_http::client::InteractionClient;
use twilight_model::id::{
    marker::{
        ApplicationMarker, ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker,
    },
    Id,
};
use uuid::Uuid;

use crate::lfg::LFGManager;

pub struct ChairContext {
    pub http: Arc<twilight_http::Client>,
    pub application_id: Id<ApplicationMarker>,
    pub cache: Arc<InMemoryCache>,
    pub latency: Latency,
    pub lfg: Arc<LFGManager>,
}

impl ChairContext {
    pub fn interaction_client(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LFGSession {
    pub uuid: Uuid,
    pub guild: Id<GuildMarker>,
    pub channel: Id<ChannelMarker>,
    pub original_message: Id<MessageMarker>,
    pub reply_message: Option<Id<MessageMarker>>,
    pub author: Id<UserMarker>,
    pub facade_tag: Id<RoleMarker>,
    pub initial_tag: Id<RoleMarker>,
    pub participants: Vec<Id<UserMarker>>,
    pub added_participants: Vec<Id<UserMarker>>,
    pub excluded_participants: Vec<Id<UserMarker>>,
    pub interested_participants: Vec<Id<UserMarker>>,
    pub initial_number: u8,
    pub required_number: u8,
    pub expiry: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChairmanUser {
    pub id: Id<UserMarker>,
    pub main_link: Option<Uuid>,
    pub linked_uuids: Option<Vec<Uuid>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub administrator: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChairmanLink {
    pub uuid: Uuid,
    pub parent: Id<UserMarker>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub last_username: String,
}
