mod repository;

use chrono::prelude::*;
use regex::Regex;
use repository::{DbFeed, Repository};
use rss::Channel;
use std::{env, error::Error, sync::{Arc, mpsc::channel, RwLock}};
use surrealdb::{
    engine::local::{Db, RocksDb},
    Surreal,
};
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db = get_db().await?;
    let repository = Repository::new(&db);

    let config = Arc::new(repository.get_config().await?);
    let feeds = repository
        .get_feeds()
        .await?
        .into_iter()
        .filter(|feed| feed.enabled);

    let sched = JobScheduler::new().await?;

    let (sender, receiver) = channel();

    for feed in feeds {
        let exec = sender.clone();
        let cron = feed.cron.clone();
        let feed = Arc::new(RwLock::new(feed));

        let feed_job = Job::new(cron.as_str(), move |_uuid, _l| {
            let feed = Arc::clone(&feed);
            exec.send(feed).unwrap()
        })
        .unwrap();
        sched.add(feed_job).await?;
    }

    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.start().await?;

    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    let space_regex = Regex::new(r"[ ]{2,}").unwrap();

    while let Ok(feed) = receiver.recv() {
        let mut feed = feed.write().unwrap();
        if let Ok(channel) = load_feed(&config.rss_proxy.as_str(), &feed).await {
            repository.mark_feed_last_run(&mut feed).await?;

            for item in channel.items() {
                println!("Id: {:?}", item.guid().unwrap());
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
        }
    }

    Ok(())
}

async fn get_db() -> Result<Surreal<Db>, Box<dyn Error>> {
    let path = env::current_dir()?;
    let path = path.join("db");

    let db = Surreal::new::<RocksDb>(path).await?;
    db.use_ns("dbo").use_db("default").await?;

    Ok(db)
}

async fn load_feed(proxy: &str, feed: &DbFeed) -> Result<Channel, Box<dyn Error>> {
    let proxy = proxy.to_owned();
    let feed_url = feed.url.as_str();
    let load_url = proxy + feed_url;

    let content = reqwest::get(load_url).await?.bytes().await?;
    let mut channel = Channel::read_from(&content[..])?;

    // filter out channel items that have pub_date after feed.last_run
    channel.items = channel
        .items()
        .into_iter()
        .filter(|item| {
            let pub_date = item.pub_date().unwrap();
            let pub_date = DateTime::parse_from_rfc2822(pub_date).unwrap();
            pub_date > feed.last_run
        })
        .cloned()
        .collect();

    Ok(channel)
}
