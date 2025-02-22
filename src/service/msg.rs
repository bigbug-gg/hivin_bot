use crate::service::Db;
use chrono::Utc;
use sqlx::Row;
use std::cmp::PartialEq;
use std::fmt::Debug;

pub struct Msg {
    conn: Db,
}

pub fn new(conn: Db) -> Msg {
    Msg { conn }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Message {
    id: i64, // 或 i32，取决于数据库字段类型
    msg_type: MsgType,
    msg_text: String,
    created_at: chrono::DateTime<Utc>, // 或其他时间类型
}

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "INTEGER")]
#[repr(i32)]
pub enum MsgType {
    Polling = 1,
    Welcome = 2,
}


impl Msg {
    pub async fn all(&self) -> Vec<Message> {
        sqlx::query("SELECT id, msg_type, msg_text, created_at FROM hv_msg")
            .map(|row: sqlx::sqlite::SqliteRow| Message {
                id: row.get("id"),
                msg_type: row.try_get("msg_type").unwrap(),
                msg_text: row.get("msg_text"),
                created_at: row.get("created_at"),
            })
            .fetch_all(&self.conn.sqlite_pool)
            .await
            .unwrap()
    }

    /// Add new msg, return the msg id.
    pub async fn add_msg(&self, msg_type: MsgType, msg_text: &str) -> i64 {
        // 欢迎消息，始终只有能有 1 条数据
        // 当存在之前的数据， 用新的数据更新
        // 不存在则新增
        if msg_type == MsgType::Welcome {
            let has_one: i32 = sqlx::query_scalar("SELECT count(*) FROM hv_msg WHERE msg_type = ?")
                .bind(msg_type.clone() as i32)
                .fetch_one(&self.conn.sqlite_pool)
                .await
                .unwrap();

            if has_one > 0 {
                let last_id = sqlx::query("UPDATE hv_msg set msg_text = ? WHERE msg_type = ?")
                    .bind(msg_text)
                    .bind(msg_type.clone() as i32)
                    .execute(&self.conn.sqlite_pool)
                    .await
                    .unwrap()
                    .last_insert_rowid();

                return last_id;
            }
        }

        sqlx::query("INSERT INTO hv_msg (msg_type, msg_text) VALUES (?, ?)")
            .bind(msg_type.clone() as i32)
            .bind(msg_text)
            .execute(&self.conn.sqlite_pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    pub async fn remove_msg(&self, id: i64) {
        sqlx::query("DELETE FROM hv_msg WHERE id = ?")
            .bind(id)
            .execute(&self.conn.sqlite_pool)
            .await
            .unwrap();
    }

    pub async fn edit_msg(&self, id: i64, msg_text: &str) -> bool {
        let rows_affected = sqlx::query("UPDATE hv_msg set msg_text = ? WHERE id = ?")
            .bind(msg_text)
            .bind(id)
            .execute(&self.conn.sqlite_pool)
            .await
            .unwrap()
            .rows_affected();
        rows_affected > 0
    }

    pub async fn welcome_msg(&self) -> String {
        sqlx::query("SELECT msg_text FROM hv_msg WHERE msg_type = ?")
            .bind(MsgType::Welcome as i32)
            .fetch_optional(&self.conn.sqlite_pool)
            .await
            .unwrap()
            .map(|row| row.get("msg_text"))
            .unwrap_or_else(|| "Hi! Nice to meet you".to_string())
    }
}
