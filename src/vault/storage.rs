use std::error::Error;

use chrono::prelude::*;
use surrealdb::{engine::local::Db, Surreal};

use super::{migration::init_db, models::{service_config::ServiceConfig, feed::DbFeed}};

#[derive(Clone)]
/// Access application data
pub struct Vault<'a> {
    db: &'a Surreal<Db>,
}

#[allow(dead_code)]
impl<'a> Vault<'a> {
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
