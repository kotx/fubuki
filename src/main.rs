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
use twilight_gateway::{Event, Intents, Shard};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_toml = tokio::fs::read_to_string("config.toml").await?;
    let config: config::Config = toml::from_str(config_toml.as_str())?;

    let mut shard = Shard::new(&config.token, Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES);
    let mut events = shard.events();

    shard.start().await?;

    let client = Client::new(&config.token);

    let subreddit = Subreddit::new(config.subreddit.as_str());

    while let Some(event) = events.next().await {
        match event {
            Event::MessageCreate(msg) if msg.content == config.trigger => {
                let mut latest = subreddit
                    .latest(50, None).await?.data.children;

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
                            .title("フブキ!")?
                            .color(0xff_00_00)?
                            .description("Sorry, I couldn't find an image!")?
                            .build()?;

                        client.create_message(msg.channel_id)
                            .embed(embed)?
                            .await?;

                        continue;
                    }
                };

                let image_url = match &post.url {
                    Some(url) => url,
                    None => {
                        continue;
                    }
                };

                let embed = EmbedBuilder::new()
                    .title("フブキ!")?
                    .color(0xff_ff_ff)?
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
