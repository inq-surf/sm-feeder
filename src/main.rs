mod data;
mod feed;
mod logger;
mod messaging;

use std::{
    error::Error,
    sync::{mpsc::channel, Arc, RwLock},
};

use tokio_cron_scheduler::{Job, JobScheduler};

use data::Vault;
use log::info;
use messaging::Broker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logger::init();

    info!("Connecting to database");
    let db = data::get_db().await?;
    let vault = Vault::new(&db);

    info!("Reading configuration");
    let config = vault.get_config().await?;
    let feeds = vault.get_active_feeds().await?;

    info!("Connecting to RabbitMQ");
    let broker = Broker::connect(&config).await?;

    // a channel to load rss feed to on cron tick
    let (job_tick, job_on_tick) = channel();

    info!("Starting scheduler");
    let scheduler = JobScheduler::new().await?;

    for feed in feeds {
        let job_tick = job_tick.clone();
        let job_cron = feed.cron.clone();
        let job_cron = job_cron.as_str();

        let feed = Arc::new(RwLock::new(feed));

        info!(
            "Scheduling job [{cron}] for feed [{name}]",
            cron = job_cron,
            name = feed.read().unwrap().name
        );
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

    while let Ok(feed) = job_on_tick.recv() {
        let mut feed = feed.write().unwrap();

        info!("Loading feed: {}", feed.url);
        if let Ok(channel) = feed::load_feed(&config.rss_proxy.as_str(), &feed).await {
            for item in channel.items() {
                let item_dto = feed::item_to_dto(item);

                vault.mark_feed_last_run(&mut feed, &item_dto.date).await?;

                info!("Publishing item: {}", item.title().unwrap());
                broker
                    .publish(
                        &config.rabbitmq_exchange,
                        &config.rabbitmq_routing_key,
                        &item_dto,
                    )
                    .await?;
            }
        }
    }

    Ok(())
}
