#![feature(drain_filter)]
#![feature(try_blocks)]

use rand::prelude::SliceRandom;
use regex::Regex;
use roux::Subreddit;
use tokio;
use futures::StreamExt;
use twilight_embed_builder::{EmbedBuilder, ImageSource};
use twilight_http::Client;
use std::error::Error;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_toml = tokio::fs::read_to_string("config.toml").await?;
    let config: config::Config = toml::from_str(config_toml.as_str())?;

    let mut shard = Shard::new(&config.token, Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES);
    let mut events = shard.some_events(
        EventTypeFlags::READY |
        EventTypeFlags::MESSAGE_CREATE
    );

    shard.start().await?;

    let client = Client::new(&config.token);

    let triggers = config.subreddits.iter()
        .map(|x| {
            x.get_trigger()
        }).collect::<Vec<_>>();

    while let Some(event) = events.next().await {
        match event {
            Event::MessageCreate(msg) if triggers.iter().any(|x| **x != "" && **x == msg.content) => {
                let group = config.subreddits.iter()
                    .filter(|x| {
                        x.get_trigger() == msg.content.as_str()
                    }).next().unwrap();

                let reddit_client = Subreddit::new(&group.subreddit.as_str());

                let mut latest = reddit_client
                    .latest(config.post_fetch_count, None).await?.data.children;

                let image_regex = Regex::new(r"https://i.redd.it/.*?\.(jpg|jpeg|gif|png)")?;

                let images = latest
                    .drain_filter(|post| {
                        let url = match &post.data.url {
                            Some(url) => url.as_str(),
                            None => "default"
                        };

                        return !post.data.over_18 && image_regex.is_match(url);
                    })
                    .collect::<Vec<_>>();

                let post = match images.choose(&mut rand::thread_rng()) {
                    Some(post) => &post.data,
                    None => {
                        let embed = EmbedBuilder::new()
                            .title(&group.title)?
                            .color(0xFF_00_00)?
                            .description(format!("Sorry, I couldn't find an image of {}", &group.title))?
                            .build()?;

                        client.create_message(msg.channel_id)
                            .embed(embed)?
                            .await?;

                        continue;
                    }
                };

                let image_url = post.url.as_ref().unwrap();

                let embed = EmbedBuilder::new()
                    .title(group.title.as_str())?
                    .color(0xff_ff_fe)?
                    .image(ImageSource::url(image_url)?)
                    .description(format!("[Sauce](https://reddit.com{})", post.permalink.as_str()))?
                    .build()?;

                client
                    .create_message(msg.channel_id)
                    .embed(embed)?
                    .await?;
            },
            Event::Ready(ready) => {
                println!("Logged in as {}#{} ({})", ready.user.name, ready.user.discriminator, ready.user.id);
            },
            _ => {},
        }
    }

    shard.shutdown();
    Ok(())
}
