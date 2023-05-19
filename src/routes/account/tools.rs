use passwords::hasher;
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::sql_entities::{account, prelude::Account};

use super::super::common::err::{Error, Result};

pub(super) async fn get_hash(cost: u8, salt: [u8; 16], password: String) -> Result<[u8; 24]> {
    tokio::task::spawn_blocking(move || hasher::bcrypt(cost, &salt, &password))
        .await
        .map_err(Error::from)?
        .map_err(|e| Error::Common(e.to_owned()))
}

pub(super) async fn try_find_account(
    sql_db: &DatabaseConnection,
    email: &String,
) -> Result<Option<account::Model>> {
    Account::find_by_id(email)
        .one(sql_db)
        .await
        .map_err(Error::from)
}
