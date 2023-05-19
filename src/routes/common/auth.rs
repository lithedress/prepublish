use std::ops::{Deref, DerefMut};

use aide::OperationIo;
use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_sessions::{
    async_session::Session,
    extractors::{ReadableSession, WritableSession},
};
use mongo::oid::ObjectId;
use serde::{Deserialize, Serialize};
use tokio::sync::OwnedRwLockWriteGuard;

use super::err::{Error, Result};

#[derive(Eq, PartialEq)]
#[derive(Copy, Clone)]
#[derive(Debug)]
#[repr(usize)]
pub(crate) enum Permission {
    Managing = 0,
    Publishing = 1,
    Nonexistent = 2,
}

#[derive(OperationIo)]
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub(crate) struct AuthInfo {
    pub(crate) id: ObjectId,
    roles: [bool; 2],
}

impl AuthInfo {
    pub(crate) fn permitted(&self, role: Permission) -> bool {
        self.roles
            .get(role as usize)
            .map(bool::to_owned)
            .unwrap_or_default()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthInfo
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        ReadableSession::from_request_parts(parts, state)
            .await
            .map_err(Error::from)?
            .get::<Self>("auth_info")
            .ok_or(Error::Forbidden("Invalid cookie!".to_string()))
    }
}

#[derive(OperationIo)]
#[derive(Debug)]
pub(crate) struct AuthInfoStorage(WritableSession);

impl AuthInfoStorage {
    pub(crate) fn store(
        &mut self,
        id: ObjectId,
        is_administrator: bool,
        is_editor: bool,
    ) -> Result<()> {
        self.0
            .insert(
                "auth_info",
                AuthInfo {
                    id,
                    roles: [is_administrator, is_editor],
                },
            )
            .map_err(Error::from)
    }
}

impl Deref for AuthInfoStorage {
    type Target = OwnedRwLockWriteGuard<Session>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AuthInfoStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthInfoStorage
where
    S: Send + Sync,
{
    type Rejection = <WritableSession as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        WritableSession::from_request_parts(parts, state)
            .await
            .map(Self)
    }
}
