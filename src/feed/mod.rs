use std::error::Error;

use chrono::DateTime;
use rss::Channel;

use crate::vault::models::feed::DbFeed;

pub async fn load_feed(proxy: &str, feed: &DbFeed) -> Result<Channel, Box<dyn Error>> {
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
