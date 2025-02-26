use crate::service::Db;
use anyhow::Result;
use sqlx::Row;

#[derive(Debug)]
pub struct PollingMsg {
    pub id: i64,
    pub hv_msg_id: i64,
    pub group_id: String,
    pub send_time: String,
    pub msg_text: String, // 从 hv_msg 表关联获取
    pub msg_title: String,
    pub msg_type: i32, // 从 hv_msg 表关联获取
}

pub struct PollingMsgDb {
    conn: Db,
}

pub fn new(conn: Db) -> PollingMsgDb {
    PollingMsgDb { conn }
}

impl PollingMsgDb {
    // 关联消息到群组
    pub async fn add_polling_msg(
        &self,
        msg_id: i64,
        group_id: &str,
        send_time: &str,
    ) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO hv_polling_msg (hv_msg_id, group_id, send_time) VALUES (?, ?, ?)",
        )
        .bind(msg_id)
        .bind(group_id)
        .bind(send_time)
        .execute(&self.conn.sqlite_pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    // 删除群组的所有关联消息
    pub async fn delete_group_msgs(&self, group_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM hv_polling_msg WHERE group_id = ?")
            .bind(group_id)
            .execute(&self.conn.sqlite_pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // 删除单条关联消息
    pub async fn delete_polling_msg_by_id(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM hv_polling_msg WHERE id = ?")
            .bind(id)
            .execute(&self.conn.sqlite_pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // 获取群组的所有关联消息
    pub async fn get_group_msgs(&self, group_id: &str) -> Result<Vec<PollingMsg>> {
        let msgs = sqlx::query(
            r#"
            SELECT pm.id, pm.hv_msg_id, g.group_id, pm.send_time, 
                   m.msg_text, m.msg_type, m.msg_title
            FROM hv_polling_msg pm
            JOIN hv_msg m ON pm.hv_msg_id = m.id
            JOIN hv_group g ON pm.group_id = g.id
            WHERE hgp.group_id = ?
            "#,
        )
        .bind(group_id)
        .map(|row: sqlx::sqlite::SqliteRow| PollingMsg {
            id: row.get("id"),
            hv_msg_id: row.get("hv_msg_id"),
            group_id: row.get("group_id"),
            send_time: row.get("send_time"),
            msg_text: row.get("msg_text"),
            msg_title: row.get("msg_title"),
            msg_type: row.get("msg_type"),
        })
        .fetch_all(&self.conn.sqlite_pool)
        .await?;

        Ok(msgs)
    }

    // 获取指定群组和时间的消息
    pub async fn get_polling_msg(
        &self,
        group_id: &str,
        send_time: &str,
    ) -> Result<Option<PollingMsg>> {
        let msg = sqlx::query(
            r#"
            SELECT pm.id, pm.hv_msg_id, g.group_id, pm.send_time,
                   m.msg_text, m.msg_type,m.msg_title
            FROM hv_polling_msg pm
            JOIN hv_msg m ON pm.hv_msg_id = m.id
        JOIN hv_group g ON pm.group_id = g.id
            WHERE pm.group_id = ? AND pm.send_time = ?
            "#,
        )
        .bind(group_id)
        .bind(send_time)
        .map(|row: sqlx::sqlite::SqliteRow| PollingMsg {
            id: row.get("id"),
            hv_msg_id: row.get("hv_msg_id"),
            group_id: row.get("group_id"),
            send_time: row.get("send_time"),
            msg_text: row.get("msg_text"),
            msg_title: row.get("msg_title"),
            msg_type: row.get("msg_type"),
        })
        .fetch_optional(&self.conn.sqlite_pool)
        .await?;

        Ok(msg)
    }

    pub async fn get_polling_msgs_by_time(
        &self,
        send_time: &str,
    ) -> Result<Vec<PollingMsg>> {
        let msgs = sqlx::query(
            r#"
        SELECT pm.id, pm.hv_msg_id, g.group_id, pm.send_time,
               m.msg_text, m.msg_type, m.msg_title
        FROM hv_polling_msg pm
        JOIN hv_msg m ON pm.hv_msg_id = m.id
        JOIN hv_group g ON pm.group_id = g.id
        WHERE pm.send_time = ?
        "#,
        )
            .bind(send_time)
            .map(|row: sqlx::sqlite::SqliteRow| PollingMsg {
                id: row.get("id"),
                hv_msg_id: row.get("hv_msg_id"),
                group_id: row.get("group_id"),
                send_time: row.get("send_time"),
                msg_text: row.get("msg_text"),
                msg_title: row.get("msg_title"),
                msg_type: row.get("msg_type"),
            })
            .fetch_all(&self.conn.sqlite_pool)
            .await?;

        Ok(msgs)
    }
}
