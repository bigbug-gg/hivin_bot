use crate::service::{polling_msg, Db};
use chrono::Utc;
use sqlx::Row;
use std::cmp::PartialEq;
use std::fmt::Debug;
use anyhow::Result;

pub struct Msg {
    conn: Db,
}

pub fn new(conn: Db) -> Msg {
    Msg { conn }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Message {
    pub id: i64, // 或 i32，取决于数据库字段类型
    pub msg_type: MsgType,
    pub msg_text: String,
    pub msg_title: String,
    pub created_at: chrono::DateTime<Utc>, // 或其他时间类型
}

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "INTEGER")]
#[repr(i32)]
pub enum MsgType {
    Polling = 1,
    Welcome = 2,
}

impl Msg {
    /// Get all message (exclude welcome type messages)
    pub async fn all(&self) -> Vec<Message> {
        sqlx::query("SELECT * FROM hv_msg WHERE msg_type = ? ")
            .bind(MsgType::Polling as i32)
            .map(|row: sqlx::sqlite::SqliteRow| Message {
                id: row.get("id"),
                msg_type: row.try_get("msg_type").unwrap(),
                msg_text: row.get("msg_text"),
                msg_title: row.get("msg_title"),
                created_at: row.get("created_at"),
            })
            .fetch_all(&self.conn.sqlite_pool)
            .await
            .unwrap()
    }

    /// Add the new message, return id.
    pub async fn add_msg(&self, msg_type: MsgType, msg_text: &str, msg_title: &str) -> i64 {
        sqlx::query("INSERT INTO hv_msg (msg_type, msg_text, msg_title) VALUES (?, ?, ?)")
            .bind(msg_type.clone() as i32)
            .bind(msg_text)
            .bind(msg_title)
            .execute(&self.conn.sqlite_pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    
    /// Add welcome message
    pub async fn add_welcome_msg(&self,msg_text: &str) -> bool {
        let msg_type = MsgType::Welcome as i32;
        let msg_title = "welcome";
        
        
        let has_one: i32 = sqlx::query_scalar("SELECT count(*) FROM hv_msg WHERE msg_type = ?")
            .bind(msg_type)
            .fetch_one(&self.conn.sqlite_pool)
            .await
            .unwrap();
        // has one do update
       if has_one > 0 {
           let rows_affected = sqlx::query("UPDATE hv_msg set msg_text = ?, msg_title = 'welcome' WHERE msg_type = ?")
                .bind(msg_text)
                .bind(msg_type)
                .execute(&self.conn.sqlite_pool)
                .await
                .unwrap()
                .rows_affected();

           return rows_affected > 0
        }
        
        let insert_id  = self.add_msg(MsgType::Welcome, msg_text, msg_title).await;
        insert_id > 0
    }

    /// Remove msg by the id
    /// 1. deleting polling data if you use this msg
    /// 2. to delete msg.
    pub async fn remove_msg(&self, msg_id: i64) -> Result<bool>{
        polling_msg::new(self.conn.clone()).delete_by_msg_id(msg_id).await?;
        
        let result_rows = sqlx::query("DELETE FROM hv_msg WHERE id = ?")
            .bind(msg_id)
            .execute(&self.conn.sqlite_pool)
            .await
            ?.rows_affected();
        Ok(result_rows > 0 )
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
