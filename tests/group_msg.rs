use hivin_bot::service::polling_msg;
use hivin_bot::service::polling_msg::PollingMsgDb;
mod common;

const MSG_ID: u32 = 3;
const GROUP_ID: &str = "22346";

async fn get_sev() -> PollingMsgDb {
    let db = common::get_db().await;
    polling_msg::new(db)
}

#[tokio::test]
async fn add_group_msg_test() {
    let sev = get_sev().await;
    let id = sev.add_polling_msg(MSG_ID as i64, GROUP_ID, "08:30").await.unwrap();
    assert!(id > 0);
    println!("group id is {}", id);
}

#[tokio::test]
async fn get_group_msg_test() {
    let sev = get_sev().await;
    let data = sev.get_polling_msg(GROUP_ID, "08:30").await.unwrap();
    assert!(data.is_some());
    println!("group id is {:?}", data.unwrap());
}