use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use chrono::Utc;
use itertools::Itertools;
use tokio::{sync::RwLock, task::AbortHandle};
use twilight_gateway::Event;
use twilight_http::{request::AuditLogReason, Client};
use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        AllowedMentions, Component, MentionType,
    },
    gateway::payload::incoming::MessageCreate,
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
    pub(crate) fn new() -> Self {
        LFGManager {
            sessions: RwLock::new(HashMap::new()),
            session_uuids: RwLock::new(HashMap::new()),
            session_timeouts: RwLock::new(HashMap::new()),
        }
    }

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

    async fn render_message(&self, client: Arc<Client>, session: LFGSession) -> Result<()> {
        let numerator = session.initial_number as usize + session.participants.len();

        let mut participants = format!("\n\n**Participants:**\n`•` <@{}>", session.author);
        participants += &session
            .participants
            .iter()
            .chain(session.added_participants.iter())
            .map(|it| format!("`•` <@{}>", it))
            .join("\n");

        let actual = session.added_participants.len() + session.participants.len() + 1;

        if actual < numerator {
            participants += &format!("\n`•` **and {} other(s)...**", numerator - actual);
        }

        participants += "\n\n*delete the original message to cancel*";

        if session.initial_number as usize + session.participants.len()
            >= session.required_number as usize
        {
            let mut mentions = format!("<@{}>", session.author);
            mentions += &session
                .participants
                .iter()
                .chain(session.added_participants.iter())
                .map(|it| format!("<@{}>", it))
                .join(" ");

            self.expire_session(client.clone(), ExpiryStrategy::DeleteOriginal, session.uuid)
                .await?;

            let embed = simple_embed(
                0x8ae24a,
                &format!(
                    "Everyone's ready! [{}/{}]",
                    numerator, session.required_number
                ),
                "Good luck everyone! Make wife proud!",
            )?;
            let embeds = &[embed];

            let allow_users_roles_mentions = &AllowedMentions {
                replied_user: false,
                parse: vec![MentionType::Roles, MentionType::Users],
                roles: vec![],
                users: vec![],
            };

            client
                .create_message(session.channel)
                .content(&format!("||{mentions}||"))
                .context("invalid message body")?
                .embeds(embeds)
                .context("invalid embed")?
                .allowed_mentions(Some(allow_users_roles_mentions))
                .await?;

            return Ok(());
        }

        let embed = simple_embed(
            0x8ae24a,
            &format!("LFG Ping [{}/{}]", numerator, session.required_number),
            &format!(
                "<@{}> is looking for a game! (expires: <t:{}:R>){}",
                session.author, session.expiry, participants
            ),
        )?;
        let embeds = &[embed];

        let component = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(format!("lfg-{}", session.uuid)),
                disabled: false,
                emoji: None,
                label: Some("Logging on / Online!".to_owned()),
                style: ButtonStyle::Primary,
                url: None,
            })],
        });

        let components = &[component];

        let reply_id = match session.reply_message {
            None => {
                let sent = client
                    .create_message(session.channel)
                    .content(&format!(
                        "{} ||{}||",
                        session.facade_tag, session.initial_tag
                    ))
                    .context("setting content")?
                    .embeds(embeds)?
                    .components(components)?
                    .await?;

                let sent_message = sent.model().await?;

                let mut sessions = self.sessions.write().await;
                let current_session = match sessions.get_mut(&session.uuid) {
                    Some(v) => v,
                    None => return Ok(()),
                };
                current_session.reply_message = Some(sent_message.id);
                drop(sessions);

                return Ok(());
            }
            Some(v) => v,
        };

        client
            .update_message(session.channel, reply_id)
            .embeds(Some(embeds))?
            .await?;

        Ok(())
    }

    pub(crate) fn on_message(event: Box<MessageCreate>) {}
}
