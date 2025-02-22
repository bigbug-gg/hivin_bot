use hivin_bot::service;
use hivin_bot::service::Db;

pub async fn get_db() -> Db {
    service::new("test.sqlite").await
}