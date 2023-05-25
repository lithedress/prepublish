use std::{net::SocketAddr, str::FromStr};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use lettre::transport::smtp::authentication::Credentials;

use mongo::MongoClient;

use crate::{cfg::AppConfig, state::AppState};

mod cfg;
mod mongo_entities;
mod routes;
mod sql_entities;
mod state;

#[tokio::main]
async fn main() {
    let config = tokio::task::spawn_blocking(AppConfig::new).await.unwrap();
    let sql_db = sea_orm::Database::connect(config.sql_db_url).await.unwrap();
    //let sql_db = sea_orm::DatabaseConnection::default();
    let mongo_db = MongoClient::with_uri_str(config.mongo_srv_url)
        .await
        .unwrap()
        .database(&config.mongo_db_nm);
    let hash_cost = config.hash_cost;
    let smtp = <AsyncSmtpTransport<Tokio1Executor>>::relay(&config.relay).unwrap().port(465).credentials(Credentials::new(config.smtp_username, config.smtp_password)).build::<Tokio1Executor>();
    assert!(smtp.test_connection().await.unwrap());
    let app = routes::new().with_state(AppState {
        sql_db,
        mongo_db,
        hash_cost,
        sender: config.sender,
        smtp,
    });
    axum::Server::bind(&SocketAddr::from_str(&config.srv_addr).unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
