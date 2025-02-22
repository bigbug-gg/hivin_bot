use hivin_bot::service::{msg::{self, Msg}};
use hivin_bot::service::msg::MsgType;

mod common;

async fn get_sev() -> Msg {
    let db = common::get_db().await;
    msg::new(db)
}

#[tokio::test]
pub async fn all_msg() {
    let ser = get_sev().await;
    let msgs = ser.all().await;
    assert!(msgs.len() > 0, "no found msg data.");
    for msg in msgs {
        println!("{:?}", msg);
    }
}

#[tokio::test]
pub async fn add_msg() {
    let ser = get_sev().await;
    let msg_type = MsgType::Welcome;
    let msg_text = "Hello, world!";
    
    let id = ser.add_msg(msg_type, msg_text).await;
    assert!(id > 0, "添加消息成功");
    println!("Add new msg success, The id: {}", id);
}

#[tokio::test]
pub async fn remove_msg() {
    let ser = get_sev().await;
    let _ = ser.remove_msg(2).await;
    let is_ok = ser.all().await.len() == 0;
    assert!(is_ok, "Remove message success");
}

#[tokio::test]
pub async fn msg_welcome() {
    let ser = get_sev().await;
    let welcome = ser.welcome_msg().await;
    println!("{:?}", welcome);
}