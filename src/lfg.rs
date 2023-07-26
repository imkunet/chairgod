use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use chrono::Utc;
use tokio::{sync::RwLock, task::AbortHandle};
use twilight_http::request::AuditLogReason;
use twilight_model::{
    channel::message::AllowedMentions,
    id::{marker::MessageMarker, Id},
};
use uuid::Uuid;

use crate::{models::LFGSession, util::simple_embed};

const EXPIRED_MESSAGES: [&str; 4] = [
    "Shoot. We left it out too long, and the ping expired",
    "Arena is dead and this unplayed ping proves it",
    "Maybe the ping would fill up if wife came back",
    "*Surely* next ping will fill up right?",
];

const BLANK_ALLOWED_MENTIONS: &AllowedMentions = &AllowedMentions {
    replied_user: false,
    parse: vec![],
    roles: vec![],
    users: vec![],
};

pub(crate) struct LFGManager {
    pub(crate) sessions: RwLock<HashMap<Uuid, LFGSession>>,
    pub(crate) session_uuids: RwLock<HashMap<Id<MessageMarker>, Uuid>>,
    pub(crate) session_timeouts: RwLock<HashMap<Uuid, AbortHandle>>,
}

#[derive(PartialEq)]
pub(crate) enum ExpiryStrategy {
    DeleteOriginal,
    ExpireMessageStale,
    ExpireMessageCancelled,
    DoNothing,
}

impl LFGManager {
    async fn expire_session(
        &self,
        client: Arc<twilight_http::Client>,
        strategy: ExpiryStrategy,
        session_id: Uuid,
    ) -> Result<()> {
        let mut timeouts = self.session_timeouts.write().await;
        // performs an abortion if there was a fetus
        if let Some(abort_handle) = timeouts.remove(&session_id) {
            abort_handle.abort()
        }
        drop(timeouts);

        let mut sessions = self.sessions.write().await;
        let session = match sessions.remove(&session_id) {
            Some(v) => v,
            None => return Ok(()),
        };
        drop(sessions);

        let mut session_uuids = self.session_uuids.write().await;
        session_uuids.remove(&session.original_message);
        drop(session_uuids);

        let reply_message = match session.reply_message {
            Some(v) => v,
            None => return Ok(()),
        };

        if strategy == ExpiryStrategy::DoNothing {
            return Ok(());
        }

        if strategy == ExpiryStrategy::DeleteOriginal {
            let delete_message = client
                .delete_message(session.channel, session.original_message)
                .reason("LFG Ping expired")
                .context("setting audit log reason")?;
            delete_message.await?;
            return Ok(());
        }

        let mut update = client
            .update_message(session.channel, reply_message)
            .content(None)
            .context("setting content to none")?
            .allowed_mentions(Some(BLANK_ALLOWED_MENTIONS));

        let embed = if strategy == ExpiryStrategy::ExpireMessageStale {
            simple_embed(
                0xff3030,
                "Expired ping",
                // feeling terrible about this one
                EXPIRED_MESSAGES
                    [(Utc::now().timestamp_millis() % EXPIRED_MESSAGES.len() as i64) as usize],
            )?
        } else {
            simple_embed(
                0xff3030,
                "Cancelled ping",
                &format!(
                    "No, that wasn't a ghost... it just looks like <@{}> backed out!",
                    session.author
                ),
            )?
        };

        let embeds = &[embed];

        update = update.embeds(Some(embeds))?;
        update.await?;

        Ok(())
    }

    async fn render_message(session: &LFGSession) {}
}
