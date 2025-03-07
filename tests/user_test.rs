
use hivin_bot::service::user::{self, User};

mod common;

async fn get_sev() -> User {
    let db = common::get_db().await;
    user::new(db)
}
#[tokio::test]
async fn add_admin() {
    let user = get_sev().await;
    let is_add = user.add_admin("12341", "xiang").await;
    assert!(is_add);
}

#[tokio::test]
async fn is_admin() {
    let user = get_sev().await;
    let is_admin = user.is_admin("12341").await;
    assert!(is_admin, "is_admin should be true {}", "12341" );
}

#[tokio::test]
async fn delete_admin() {
    let user = get_sev().await;
    let is_delete = user.delete_admin("6762311321").await;
    assert!(is_delete);
}

#[tokio::test]
async fn all_admin() {
    let user = get_sev().await;
    let admins = user.all_admins().await;
    assert!(!admins.is_empty());
    for admin in admins {
        println!("{:?}", admin);
    }
}