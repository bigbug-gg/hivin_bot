use hivin_bot::service::group;
use hivin_bot::service::group::Group;

mod common;

const GROUP_ID: &str = "22346";
const GROUP_NAME: &str = "李四";
async fn get_sev() -> Group {
    let db = common::get_db().await;
    group::new(db)
}

#[tokio::test]
async fn add_test() {
    let sev = get_sev().await;
    let id = sev.add_group(GROUP_ID, GROUP_NAME).await.unwrap();
    assert!(id > 0, "add_group return id should not be 0");
    println!("add_group return {:?}", id);
}

#[tokio::test]
async fn all_test() {
    let sev = get_sev().await;
    let list = sev.all().await;
    assert!(list.len() > 0, "list should is empty");
    for item in list {
        println!("item {:?}", item);
    }
}

#[tokio::test]
async fn set_mute_test() {
    let sev = get_sev().await;
    let _ = sev.set_mute_polling(GROUP_ID, false).await.unwrap();
    let _ = sev.set_mute_welcome(GROUP_ID, false).await.unwrap();
    
    let is_mute_polling = sev.should_do_polling(GROUP_ID).await.unwrap();
    let is_mute_welcome = sev.should_do_welcome(GROUP_ID).await.unwrap();
    assert_eq!(is_mute_polling, false, "set_mute_polling should not be false, got {:?}", is_mute_polling);
    assert_eq!(is_mute_welcome, false,  "set_mute_welcome should not be false, got {:?}", is_mute_welcome);
}