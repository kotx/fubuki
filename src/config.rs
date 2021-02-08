use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub trigger: String,
    pub subreddit: String,
}
