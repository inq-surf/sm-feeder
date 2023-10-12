use chrono::prelude::*;
use regex::Regex;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::{env, error::Error};
use surrealdb::{engine::local::RocksDb, Surreal};

#[derive(Serialize, Deserialize, Debug)]
struct ServiceConfig {
    created: DateTime<Utc>,
    rss_proxy: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    let space_regex = Regex::new(r"[ ]{2,}").unwrap();

    load_config().await?;

    let feed = example_feed().await;
    feed.map(|channel| {
        for item in channel.items() {
            println!("Title: {}", item.title().unwrap());
            println!("Link: {}", item.link().unwrap());
            println!(
                "Description: {}\n",
                space_regex.replace_all(
                    &tag_regex.replace_all(item.description().unwrap(), r" "),
                    r"\n"
                )
            );
        }
    })
    .unwrap_or_else(|err| {
        println!("error: {}", err);
    });
    println!("Hello, world!");

    Ok(())
}

async fn load_config() -> Result<ServiceConfig, Box<dyn Error>> {
    let path = env::current_dir().unwrap();
    let db = Surreal::new::<RocksDb>(path.join("db")).await?;
    db.use_ns("dbo").use_db("default").await?;

    let mut config: Option<ServiceConfig> = db
        .query("select * from config order by created desc limit 1")
        .await?
        .take(0)?;

    if config.is_none() {
        config = db
            .create("config")
            .content(ServiceConfig {
                created: Utc::now(),
                rss_proxy: "https://rss.x.qrd.wtf/makefulltextfeed.php?url=".to_string(),
            })
            .await?
            .into_iter()
            .next();
    }

    println!("config: {:?}", config);

    Ok(config.unwrap())
}

async fn example_feed() -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(
        "https://rss.x.qrd.wtf/makefulltextfeed.php?url=http://feeds.bbci.co.uk/news/world/rss.xml",
    )
    .await?
    .bytes()
    .await?;
    let channel = Channel::read_from(&content[..])?;

    Ok(channel)
}
