use sqlx::{migrate, SqlitePool, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use xmpp_parsers::Jid;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> crate::error::Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Run migrations
        migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { pool })
    }

    // Account operations
    pub async fn save_account(&self, jid: &Jid, name: &str) -> crate::error::Result<()> {
        sqlx::query!(
            "INSERT OR REPLACE INTO accounts (jid, name, created_at) VALUES (?, ?, ?)",
            jid.to_string(),
            name,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_accounts(&self) -> crate::error::Result<Vec<(String, String, DateTime<Utc>)>> {
        let rows = sqlx::query!(
            "SELECT jid, name, created_at FROM accounts ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter()
            .map(|row| (row.jid, row.name, row.created_at))
            .collect())
    }

    // Message operations
    pub async fn save_message(
        &self,
        from_jid: &Jid,
        to_jid: &Jid,
        body: &str,
        message_type: &str,
        stanza_id: &str,
    ) -> crate::error::Result<String> {
        let id = Uuid::new_v4().to_string();
        
        sqlx::query!(
            "INSERT INTO messages (id, from_jid, to_jid, body, message_type, stanza_id, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            id,
            from_jid.to_string(),
            to_jid.to_string(),
            body,
            message_type,
            stanza_id,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_chat_history(
        &self,
        user_jid: &Jid,
        contact_jid: &Jid,
        limit: i64,
        offset: i64,
    ) -> crate::error::Result<Vec<ChatMessage>> {
        let user_jid_str = user_jid.to_string();
        let contact_jid_str = contact_jid.to_string();

        let rows = sqlx::query!(
            "SELECT id, from_jid, to_jid, body, message_type, stanza_id, created_at 
             FROM messages 
             WHERE (from_jid = ? AND to_jid = ?) OR (from_jid = ? AND to_jid = ?)
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?",
            user_jid_str,
            contact_jid_str,
            contact_jid_str,
            user_jid_str,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter()
            .map(|row| ChatMessage {
                id: row.id,
                from_jid: row.from_jid,
                to_jid: row.to_jid,
                body: row.body,
                message_type: row.message_type,
                stanza_id: row.stanza_id,
                created_at: row.created_at,
            })
            .collect())
    }

    // Roster operations
    pub async fn add_roster_item(
        &self,
        user_jid: &Jid,
        contact_jid: &Jid,
        name: Option<&str>,
        groups: &[String],
    ) -> crate::error::Result<()> {
        let contact_jid_str = contact_jid.to_string();
        let user_jid_str = user_jid.to_string();

        sqlx::query!(
            "INSERT OR REPLACE INTO roster_items (user_jid, contact_jid, name, subscription, created_at) 
             VALUES (?, ?, ?, ?, ?)",
            user_jid_str,
            contact_jid_str,
            name,
            "none",
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        // Add groups
        for group in groups {
            sqlx::query!(
                "INSERT OR REPLACE INTO roster_groups (user_jid, contact_jid, group_name) VALUES (?, ?, ?)",
                user_jid_str,
                contact_jid_str,
                group
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_roster(&self, user_jid: &Jid) -> crate::error::Result<Vec<RosterItem>> {
        let user_jid_str = user_jid.to_string();

        let rows = sqlx::query!(
            "SELECT r.contact_jid, r.name, r.subscription, r.created_at 
             FROM roster_items r 
             WHERE r.user_jid = ? 
             ORDER BY r.name, r.contact_jid",
            user_jid_str
        )
        .fetch_all(&self.pool)
        .await?;

        let mut roster_items = Vec::new();
        for row in rows {
            let groups = sqlx::query!(
                "SELECT group_name FROM roster_groups 
                 WHERE user_jid = ? AND contact_jid = ?",
                user_jid_str,
                row.contact_jid
            )
            .fetch_all(&self.pool)
            .await?;

            roster_items.push(RosterItem {
                jid: row.contact_jid,
                name: row.name,
                subscription: row.subscription,
                groups: groups.into_iter().map(|g| g.group_name).collect(),
                created_at: row.created_at,
            });
        }

        Ok(roster_items)
    }

    // Presence operations
    pub async fn update_presence(
        &self,
        jid: &Jid,
        show: &str,
        status: Option<&str>,
    ) -> crate::error::Result<()> {
        sqlx::query!(
            "INSERT OR REPLACE INTO presence (jid, show, status, updated_at) VALUES (?, ?, ?, ?)",
            jid.to_string(),
            show,
            status,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_presence(&self, jid: &Jid) -> crate::error::Result<Option<Presence>> {
        let row = sqlx::query!(
            "SELECT jid, show, status, updated_at FROM presence WHERE jid = ?",
            jid.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Presence {
            jid: r.jid,
            show: r.show,
            status: r.status,
            updated_at: r.updated_at,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub from_jid: String,
    pub to_jid: String,
    pub body: String,
    pub message_type: String,
    pub stanza_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RosterItem {
    pub jid: String,
    pub name: Option<String>,
    pub subscription: String,
    pub groups: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Presence {
    pub jid: String,
    pub show: String,
    pub status: Option<String>,
    pub updated_at: DateTime<Utc>,
}