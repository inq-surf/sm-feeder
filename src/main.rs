mod feed;
mod vault;

use std::{
    error::Error,
    sync::{mpsc::channel, Arc, RwLock},
};

use feed::load_feed;
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use regex::Regex;
use vault::{storage::Vault, connection::get_db};
use serde::Serialize;

use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Serialize)]
struct SerializedItem {
    guid: String,
    title: String,
    link: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db = get_db().await?;
    let repository = Vault::new(&db);

    let config = repository.get_config().await?;
    let feeds = repository.get_active_feeds().await?;

    let mq = Connection::connect(
        config.rabbitmq_uri.as_str(),
        ConnectionProperties::default(),
    )
    .await?;

    // a channel to load rss feed to on cron tick
    let (job_tick, job_on_tick) = channel();

    let scheduler = JobScheduler::new().await?;

    for feed in feeds {
        let job_tick = job_tick.clone();
        let job_cron = feed.cron.clone();
        let job_cron = job_cron.as_str();

        let feed = Arc::new(RwLock::new(feed));

        let feed_job = Job::new(job_cron, move |_uuid, _l| {
            let feed = Arc::clone(&feed);

            job_tick.send(feed).unwrap()
        })
        .unwrap();

        scheduler.add(feed_job).await?;
    }

    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    scheduler.start().await?;

    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    let space_regex = Regex::new(r"[ ]{2,}").unwrap();

    let mq_channel = mq.create_channel().await?;
    mq_channel
        .exchange_declare(
            config.rabbitmq_exchange.as_str(),
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            Default::default(),
        )
        .await?;

    while let Ok(feed) = job_on_tick.recv() {
        let mut feed = feed.write().unwrap();

        if let Ok(channel) = load_feed(&config.rss_proxy.as_str(), &feed).await {
            repository.mark_feed_last_run(&mut feed).await?;

            for item in channel.items() {
                let serialized_item = serde_json::to_vec(&SerializedItem {
                    guid: item.guid().unwrap().value().to_string(),
                    title: item.title().unwrap().to_string(),
                    link: item.link().unwrap().to_string(),
                    description: space_regex
                        .replace_all(
                            &tag_regex.replace_all(item.description().unwrap(), r" "),
                            r"\n",
                        )
                        .to_string(),
                })
                .unwrap();

                mq_channel
                    .basic_publish(
                        &config.rabbitmq_exchange,
                        &config.rabbitmq_routing_key,
                        BasicPublishOptions::default(),
                        &serialized_item,
                        BasicProperties::default(),
                    )
                    .await?;
            }
        }
    }

    Ok(())
}
