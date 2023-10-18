use std::error::Error;

use chrono::DateTime;
use regex::Regex;
use rss::Channel;

use crate::data::models::Feed;

pub async fn load_feed(proxy: &str, feed: &Feed) -> Result<Channel, Box<dyn Error>> {
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
        .map(|item| {
            let mut item = item.clone();

            item.description = match item.description() {
                Some(description) => Some(clear_tags(&description)),
                None => return item,
            };

            item
        })
        .collect();

    Ok(channel)
}

fn clear_tags(content: &str) -> String {
    // TODO: do not recreate regexes on every call
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    let space_regex = Regex::new(r"[ ]{2,}").unwrap();

    let content = tag_regex.replace_all(content, " ");
    space_regex.replace_all(&content, "\n").to_string()
}
