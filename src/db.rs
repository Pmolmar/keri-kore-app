use keri_core::timestamped::TimestampedSignedEventMessage;
use keri_core::{
    database::timestamped::TimestampedSignedEventMessage,
    database::{EventDatabase, QueryParameters},
    event_message::signed_event_message::{
        SignedEventMessage, SignedNontransferableReceipt, SignedTransferableReceipt,
    },
    prefix::IdentifierPrefix,
};
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::sync::Arc;

pub struct SqlxEventDatabase {
    pool: Arc<Pool<Sqlite>>,
}

#[derive(Debug)]
pub enum SqlxDatabaseError {
    DatabaseError(sqlx::Error),
    SerializationError(serde_json::Error),
}

impl SqlxEventDatabase {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;

        // Create necessary tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kel_events (
                sn INTEGER PRIMARY KEY AUTOINCREMENT
                id TEXT NOT NULL,
                event_data TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS receipts_t (
            sn INTEGER PRIMARY KEY AUTOINCREMENT
                id TEXT NOT NULL,
                receipt_data TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}

impl EventDatabase for SqlxEventDatabase {
    type Error = SqlxDatabaseError;

    fn add_kel_finalized_event(
        &self,
        event: SignedEventMessage,
        id: &IdentifierPrefix,
    ) -> Result<(), Self::Error> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        let event_json =
            serde_json::to_string(&event).map_err(SqlxDatabaseError::SerializationError)?;

        tokio::spawn(async move {
            sqlx::query("INSERT INTO kel_events (id, event_data) VALUES (?, ?)")
                .bind(id_str)
                .bind(event_json)
                .execute(&*pool)
                .await
                .map_err(SqlxDatabaseError::DatabaseError)
        });

        Ok(())
    }

    fn add_receipt_t(
        &self,
        receipt: SignedTransferableReceipt,
        id: &IdentifierPrefix,
    ) -> Result<(), Self::Error> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        let receipt_json =
            serde_json::to_string(&receipt).map_err(SqlxDatabaseError::SerializationError)?;

        tokio::spawn(async move {
            sqlx::query("INSERT INTO receipts_t (id, receipt_data) VALUES (?, ?)")
                .bind(id_str)
                .bind(receipt_json)
                .execute(&*pool)
                .await
                .map_err(SqlxDatabaseError::DatabaseError)
        });

        Ok(())
    }

    fn add_receipt_nt(
        &self,
        receipt: SignedNontransferableReceipt,
        id: &IdentifierPrefix,
    ) -> Result<(), Self::Error> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        let receipt_json =
            serde_json::to_string(&receipt).map_err(SqlxDatabaseError::SerializationError)?;

        tokio::spawn(async move {
            sqlx::query("INSERT INTO receipts_nt (id, receipt_data) VALUES (?, ?)")
                .bind(id_str)
                .bind(receipt_json)
                .execute(&*pool)
                .await
                .map_err(SqlxDatabaseError::DatabaseError)
        });

        Ok(())
    }

    fn get_kel_finalized_events(
        &self,
        params: QueryParameters,
    ) -> Option<impl DoubleEndedIterator<Item = TimestampedSignedEventMessage>> {
        let pool = self.pool.clone();

        // Convert QueryParameters to SQL query conditions
        let query = match params {
            QueryParameters::BySn { id, sn } => {
                "SELECT event_data, timestamp FROM kel_events WHERE id = ? AND sn = ? ORDER BY timestamp"
            }
            QueryParameters::Range { id, start, limit } => {
                "SELECT event_data, timestamp FROM kel_events WHERE id = ? AND timestamp >= ? ORDER BY timestamp"
            }
            QueryParameters::All { id } => {
                "SELECT event_data, timestamp FROM kel_events WHERE id = ? ORDER BY timestamp"
            }
        };

        tokio::spawn(async move {
            sqlx::query("INSERT INTO receipts_nt (id, receipt_data) VALUES (?, ?)")
                .bind(id_str)
                .bind(receipt_json)
                .execute(&*pool)
                .await
                .map_err(SqlxDatabaseError::DatabaseError)
        });

        // Execute query and transform results
        let events = tokio::spawn(async move {
            let rows = sqlx::query(query).fetch_all(&*pool).await.ok()?;

            let events: Vec<TimestampedSignedEventMessage> = rows
                .into_iter()
                .filter_map(|row| {
                    let event_data: String = row.get("event_data");
                    let timestamp: DateTime<Utc> = row.get("timestamp");

                    serde_json::from_str(&event_data)
                        .map(|event| TimestampedSignedEventMessage { event, timestamp })
                        .ok()
                })
                .collect();

            Some(events.into_iter())
        });

        events.ok()?
    }

    fn get_receipts_t(
        &self,
        params: QueryParameters,
    ) -> Option<impl DoubleEndedIterator<Item = Transferable>> {
        let pool = self.pool.clone();

        let query = match params {
            QueryParameters::All => "SELECT receipt_data FROM receipts_t ORDER BY timestamp",
            QueryParameters::Id(id) => {
                "SELECT receipt_data FROM receipts_t WHERE id = ? ORDER BY timestamp"
            } // Add other cases as needed
        };

        let receipts = tokio::spawn(async move {
            let rows = sqlx::query(query).fetch_all(&*pool).await.ok()?;

            let receipts: Vec<Transferable> = rows
                .into_iter()
                .filter_map(|row| {
                    let receipt_data: String = row.get("receipt_data");
                    serde_json::from_str(&receipt_data).ok()
                })
                .collect();

            Some(receipts.into_iter())
        });

        receipts.ok()?
    }

    fn get_receipts_nt(
        &self,
        params: QueryParameters,
    ) -> Option<impl DoubleEndedIterator<Item = SignedNontransferableReceipt>> {
        let pool = self.pool.clone();

        let query = match params {
            QueryParameters::All => "SELECT receipt_data FROM receipts_nt ORDER BY timestamp",
            QueryParameters::Id(id) => {
                "SELECT receipt_data FROM receipts_nt WHERE id = ? ORDER BY timestamp"
            } // Add other cases as needed
        };

        let receipts = tokio::spawn(async move {
            let rows = sqlx::query(query).fetch_all(&*pool).await.ok()?;

            let receipts: Vec<SignedNontransferableReceipt> = rows
                .into_iter()
                .filter_map(|row| {
                    let receipt_data: String = row.get("receipt_data");
                    serde_json::from_str(&receipt_data).ok()
                })
                .collect();

            Some(receipts.into_iter())
        });

        receipts.ok()?
    }
}
