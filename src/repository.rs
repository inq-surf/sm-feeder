use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, error::Error};
use surrealdb::{engine::local::Db, Surreal};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServiceConfig {
    pub created: DateTime<Utc>,
    pub rss_proxy: String,
    pub rabbitmq_uri: String,
    pub rabbitmq_exchange: String,
    pub rabbitmq_routing_key: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DbFeed {
    pub url: String,
    pub name: String,
    pub cron: String,
    pub enabled: bool,
    pub last_run: DateTime<Utc>,
}

#[derive(Clone)]
/// Repository to access application data
pub struct Repository<'a> {
    db: &'a Surreal<Db>,
}

#[allow(dead_code)]
impl<'a> Repository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Get the application config
    pub async fn get_config(&self) -> Result<ServiceConfig, Box<dyn Error>> {
        let mut config: Option<ServiceConfig> = self
            .db
            .query("select * from config order by created desc limit 1")
            .await?
            .take(0)?;

        if config.is_none() {
            config = Some(init_db(&self.db).await?);
        }

        let config = config.ok_or("No config found")?;

        Ok(config)
    }

    /// Get all feeds
    pub async fn get_feeds(&self) -> Result<Vec<DbFeed>, Box<dyn Error>> {
        let feeds: Vec<DbFeed> = self.db.select("feed").await?;

        Ok(feeds)
    }

    pub async fn get_active_feeds(&self) -> Result<Vec<DbFeed>, Box<dyn Error>> {
        let feeds: Vec<DbFeed> = self
            .get_feeds()
            .await?
            .into_iter()
            .filter(|feed| feed.enabled)
            .collect();

        Ok(feeds)
    }

    pub async fn set_feed_name(&self, feed: &DbFeed) -> Result<(), Box<dyn Error>> {
        self.db
            .query("update feed set name = $name where url = $url")
            .bind(("url", feed.url.clone()))
            .bind(("name", feed.name.clone()))
            .await?;

        Ok(())
    }

    pub async fn enable_feed(&self, feed: &mut DbFeed) -> Result<(), Box<dyn Error>> {
        feed.enabled = true;

        self.db
            .query("update feed set enabled = true where url = $url")
            .bind(("url", feed.url.clone()))
            .await?;

        Ok(())
    }

    pub async fn disable_feed(&self, feed: &mut DbFeed) -> Result<(), Box<dyn Error>> {
        feed.enabled = false;

        self.db
            .query("update feed set enabled = false where url = $url")
            .bind(("url", feed.url.clone()))
            .await?;

        Ok(())
    }

    pub async fn mark_feed_last_run(&self, feed: &mut DbFeed) -> Result<(), Box<dyn Error>> {
        feed.last_run = Utc::now();

        self.db
            .query("update feed set last_run = $last_run where url = $url")
            .bind(("url", feed.url.clone()))
            .bind(("last_run", feed.last_run))
            .await?;

        Ok(())
    }
}

/// Initialize the database with default values
///
/// Returns the config
async fn init_db(db: &Surreal<Db>) -> Result<ServiceConfig, Box<dyn Error>> {
    let config = db
        .create("config")
        .content(ServiceConfig {
            created: Utc::now(),
            rss_proxy: env::var("INIT_RSS_PROXY").unwrap_or(String::from(
                "http://ftr.fivefilters.org/makefulltextfeed.php?url=",
            )),
            rabbitmq_uri: env::var("INIT_RABBITMQ_URI")
                .unwrap_or(String::from("amqp://guest:guest@localhost:5672/%2f")),
            rabbitmq_exchange: env::var("INIT_RABBITMQ_EXCHANGE").unwrap_or(String::from("rss")),
            rabbitmq_routing_key: env::var("INIT_RABBITMQ_ROUTING_KEY")
                .unwrap_or(String::from("inbox")),
        })
        .await?
        .into_iter()
        .next()
        .unwrap();

    match env::var("INIT_RSS_FEEDS") {
        Ok(feeds) => {
            for feed in feeds.split(";") {
                let feed = feed.trim();
                let feed = feed.split(",").collect::<Vec<&str>>();

                let url = feed[0];
                let name = feed[1];
                let cron = feed[2];

                let _: Vec<DbFeed> = db
                    .create("feed")
                    .content(DbFeed {
                        url: String::from(url),
                        name: String::from(name),
                        cron: String::from(cron),
                        enabled: true,
                        last_run: Utc::now(),
                    })
                    .await?;
            }
        }
        Err(_) => {
            let _: Vec<DbFeed> = db
                .create("feed")
                .content(DbFeed {
                    url: String::from("http://feeds.bbci.co.uk/news/world/rss.xml"),
                    name: String::from("BBC News"),
                    cron: String::from("0 */5 * * * *"),
                    enabled: true,
                    last_run: Utc::now(),
                })
                .await?;
        }
    }

    Ok(config)
}
