use lettre::{AsyncSmtpTransport, Tokio1Executor};
use lettre::message::Mailbox;
use mongo::MongoDatabase;
use sea_orm::DatabaseConnection;

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) sql_db: DatabaseConnection,
    pub(crate) mongo_db: MongoDatabase,
    pub(crate) hash_cost: u8,
    pub(crate) sender: Mailbox,
    pub(crate) smtp: AsyncSmtpTransport<Tokio1Executor>,
}
