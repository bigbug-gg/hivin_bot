use crate::service::Db;
use chrono::Utc;
use sqlx::Row;
use anyhow::Result;

pub struct Group {
    conn: Db,
}

#[derive(sqlx::FromRow, Debug)]
pub struct GroupInfo {
    pub id: i64,
    pub group_id: String,
    pub group_name: String,
    pub mute_polling: bool,
    pub mute_welcome: bool,
    pub created_at: chrono::DateTime<Utc>,
}

pub fn new(conn: Db) -> Group {
    Group { conn }
}

impl Group {
    pub async fn all(&self) -> Vec<GroupInfo> {
        sqlx::query("SELECT * FROM hv_group")
            .map(|row: sqlx::sqlite::SqliteRow| GroupInfo {
                id: row.get("id"),
                group_id: row.get("group_id"),
                group_name: row.get("group_name"),
                mute_polling: row.get("mute_polling"),
                mute_welcome: row.get("mute_welcome"),
                created_at: row.get("created_at"),
            })
            .fetch_all(&self.conn.sqlite_pool)
            .await
            .unwrap()
    }

    pub async fn get_by_id(&self, id: i64) -> Option<GroupInfo> {
        sqlx::query_as::<_, GroupInfo>("SELECT * FROM hv_group WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.conn.sqlite_pool)
            .await
            .unwrap()
    }

    pub async fn add_group(&self, group_id: &str, group_name: &str) -> Result<i64> {
        // 先查询
        let existing = sqlx::query("SELECT id FROM hv_group WHERE group_id = ?")
            .bind(group_id)
            .fetch_optional(&self.conn.sqlite_pool)
            .await?;

        // 如果存在返回已有id,否则插入新记录
        match existing {
            Some(row) => Ok(row.get("id")),
            None => {
                let result = sqlx::query("
                INSERT INTO hv_group (group_id, group_name, mute_polling, mute_welcome) 
                VALUES (?, ?, ?, ?)
            ")
                    .bind(group_id)
                    .bind(group_name)
                    .bind(true)
                    .bind(true)
                    .execute(&self.conn.sqlite_pool)
                    .await?;

                Ok(result.last_insert_rowid())
            }
        }
    }

    pub async fn set_mute_polling(&self, group_id: &str, mute: bool) -> Result<()> {
        sqlx::query("
        UPDATE hv_group 
        SET mute_polling = ? 
        WHERE group_id = ?
    ")
            .bind(mute)
            .bind(group_id)
            .execute(&self.conn.sqlite_pool)
            .await?;

        Ok(())
    }

    pub async fn set_mute_welcome(&self, group_id: &str, mute: bool) -> Result<()> {
        sqlx::query("
        UPDATE hv_group 
        SET mute_welcome = ? 
        WHERE group_id = ?
    ")
            .bind(mute)
            .bind(group_id)
            .execute(&self.conn.sqlite_pool)
            .await?;

        Ok(())
    }


    pub async fn should_do_polling(&self, group_id: &str) -> Result<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            "SELECT mute_polling FROM hv_group WHERE group_id = ?"
        )
            .bind(group_id)
            .fetch_optional(&self.conn.sqlite_pool)
            .await?;

        match result {
            Some((mute_polling,)) => Ok(mute_polling),
            None => Ok(false)
        }
    }

    pub async fn should_do_welcome(&self, group_id: &str) -> Result<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            "SELECT mute_welcome FROM hv_group WHERE group_id = ?"
        )
            .bind(group_id)
            .fetch_optional(&self.conn.sqlite_pool)
            .await?;

        match result {
            Some((mute_welcome,)) => Ok(mute_welcome),
            None => Ok(false)
        }
    }

    pub async fn delete_group(&self, group_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM hv_group WHERE group_id = ?"
        )
            .bind(group_id)
            .execute(&self.conn.sqlite_pool)
            .await?;

      
        Ok(result.rows_affected() > 0)
    }
}
