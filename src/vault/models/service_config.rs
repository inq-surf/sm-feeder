use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServiceConfig {
    pub created: DateTime<Utc>,
    pub rss_proxy: String,
    pub rabbitmq_uri: String,
    pub rabbitmq_exchange: String,
    pub rabbitmq_routing_key: String,
}
