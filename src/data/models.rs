use chrono::{DateTime, Utc, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub created: DateTime<Utc>,
    pub rss_proxy: String,
    pub rabbitmq_uri: String,
    pub rabbitmq_exchange: String,
    pub rabbitmq_routing_key: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Feed {
    pub url: String,
    pub name: String,
    pub cron: String,
    pub enabled: bool,
    pub last_run: DateTime<FixedOffset>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FeedItemDto {
    pub guid: String,
    pub title: String,
    pub link: String,
    pub date: DateTime<FixedOffset>,
    pub description: String,
}
