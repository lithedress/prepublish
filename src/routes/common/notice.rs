use crate::mongo_entities::profile::Profile;
use crate::state::AppState;
use lettre::message::{Mailbox, MaybeString, MessageBuilder};
use lettre::AsyncTransport;
use mongo::entity::Entity;

pub(crate) async fn send_email(
    state: AppState,
    receiver_profile: Entity<Profile>,
    subject: impl Into<String>,
    body: impl Into<String>,
) {
    state
        .smtp
        .send(
            MessageBuilder::new()
                .sender(state.sender)
                .to(Mailbox::new(
                    Some(receiver_profile.data.bio.name),
                    receiver_profile.data.email,
                ))
                .subject(subject)
                .body(MaybeString::String(body.into()))
                .unwrap(),
        )
        .await
        .unwrap();
}
