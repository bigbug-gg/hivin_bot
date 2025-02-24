pub mod user;
pub mod msg;
pub mod group;
pub mod polling_msg;

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Db { sqlite_pool: SqlitePool }


pub async fn new(path: &str) -> Db {
    let sqlite_pool = SqlitePool::connect(format!("sqlite:{path}?mode=rwc").as_str()).await.unwrap();
    init_db(&sqlite_pool).await;
    Db { sqlite_pool }
}



/// 初始表
/// hv_user 存储管理员
/// hv_msg 设置消息
/// hv_group 机器人加入的群
/// hv_polling_msg 群定时推送消息设置
async fn init_db(conn: &SqlitePool) -> bool {
    // user table
    let _ = sqlx::query(
        "
CREATE TABLE IF NOT EXISTS hv_user (
id INTEGER PRIMARY KEY AUTOINCREMENT,
user_id VARCHAR(32) NOT NULL,
user_name VARCHAR(32) NOT NULL,
is_admin BOOLEAN DEFAULT FALSE,
created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);

CREATE TABLE IF NOT EXISTS hv_msg (
id INTEGER PRIMARY KEY AUTOINCREMENT,
msg_title VARCHAR(32) DEFAULT '',
msg_type INTEGER NOT NULL DEFAULT 1,
msg_text TEXT NOT NULL,
created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);

CREATE TABLE IF NOT EXISTS hv_group (
id INTEGER PRIMARY KEY AUTOINCREMENT,
group_id VARCHAR(32) NOT NULL,
group_name VARCHAR(32) NOT NULL,
mute_polling BOOLEAN DEFAULT FALSE,
mute_welcome BOOLEAN DEFAULT FALSE,
created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);

CREATE TABLE IF NOT EXISTS hv_tele_group (
id INTEGER PRIMARY KEY AUTOINCREMENT,
group_id VARCHAR(32) NOT NULL,
group_name VARCHAR(32) NOT NULL,
mute_polling BOOLEAN DEFAULT FALSE,
mute_welcome BOOLEAN DEFAULT FALSE,
created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);

CREATE TABLE IF NOT EXISTS hv_polling_msg (
id INTEGER PRIMARY KEY AUTOINCREMENT,
hv_msg_id INTEGER NOT NULL,
group_id VARCHAR(32) NOT NULL,
send_time VARCHAR(8) NOT NULL,
created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);
",
    )
    .execute(conn)
    .await
    .unwrap();
    true
}
