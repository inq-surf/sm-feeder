use std::{env, error::Error};

use chrono::Utc;
use surrealdb::{engine::local::Db, Surreal};

use super::models::{service_config::ServiceConfig, feed::DbFeed};

/// Initialize the database with default values
///
/// Returns the config
pub async fn init_db(db: &Surreal<Db>) -> Result<ServiceConfig, Box<dyn Error>> {
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
