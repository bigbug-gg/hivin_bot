use crate::service::Db;
use chrono::Utc;
use sqlx::Row;

pub struct User {
    conn: Db,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Admin {
    pub id: i64, // 或 i32，取决于数据库字段类型
    pub user_id: String,
    pub user_name: String,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<Utc>, // 或其他时间类型
}

pub fn new(conn: Db) -> User {
    User { conn }
}

impl User {
    pub async fn has_admin(&self) -> bool {
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM hv_user WHERE is_admin = 1")
            .fetch_one(&self.conn.sqlite_pool)
            .await
            .unwrap_or(0);
        count > 0
    }
    
    pub async fn is_admin(&self, user_id: &str) -> bool {
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM hv_user WHERE user_id = ? AND is_admin = 1")
            .bind(user_id)
            .fetch_one(&self.conn.sqlite_pool)
            .await
            .unwrap();

        count > 0
    }

    /// Add admin
    pub async fn add_admin(&self, user_id: &str, user_name: &str) -> bool {
        let result = sqlx::query(
            "
        INSERT OR IGNORE INTO hv_user (user_id, user_name, is_admin)
        SELECT ?, ?, ?
        WHERE NOT EXISTS (
            SELECT 1 FROM hv_user WHERE user_id = ?
        );
    ",
        )
        .bind(user_id)
        .bind(user_name)
        .bind(true)
        .bind(user_id)
        .execute(&self.conn.sqlite_pool)
        .await
        .unwrap();

        result.rows_affected() > 0
    }

    /// Delete admin
    pub async fn delete_admin(&self, user_id: &str) -> bool {
        let result = sqlx::query(
            "
        DELETE FROM hv_user WHERE user_id = ?
        ",
        )
        .bind(user_id)
        .execute(&self.conn.sqlite_pool)
        .await
        .unwrap();
        result.rows_affected() > 0
    }

    pub async fn all_admins(&self) -> Vec<Admin> {
        sqlx::query("SELECT id, user_id, user_name, is_admin, created_at FROM hv_user")
            .map(|row: sqlx::sqlite::SqliteRow| Admin {
                id: row.get("id"),
                user_id: row.get("user_id"),
                user_name: row.get("user_name"),
                is_admin: row.get("is_admin"),
                created_at: row.get("created_at"),
            })
            .fetch_all(&self.conn.sqlite_pool)
            .await
            .unwrap()
    }

    pub async fn cancel_admin(&self, user_id: &str) -> bool {
        let result = sqlx::query(
            "UPDATE hv_user set is_admin = false WHERE user_id = ? and is_admin = true",
        )
        .bind(user_id)
        .execute(&self.conn.sqlite_pool)
        .await
        .unwrap();
        result.rows_affected() > 0
    }

    pub async fn set_admin(&self, user_id: &str) -> bool {
        let result = sqlx::query(
            "UPDATE hv_user set is_admin = true WHERE user_id = ? and is_admin = false",
        )
        .bind(user_id)
        .execute(&self.conn.sqlite_pool)
        .await
        .unwrap();
        result.rows_affected() > 0
    }

    pub async fn set_admin_name(&self, user_id: &str, name: &str) -> bool {
        let result = sqlx::query(
            "UPDATE hv_user set name = ? WHERE user_id = ?"
        ).bind(name).bind(user_id).execute(&self.conn.sqlite_pool).await.unwrap().rows_affected();
        result > 0
    }
}
