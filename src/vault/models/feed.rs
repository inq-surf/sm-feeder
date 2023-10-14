use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DbFeed {
    pub url: String,
    pub name: String,
    pub cron: String,
    pub enabled: bool,
    pub last_run: DateTime<Utc>,
}
