use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
    #[serde(default = "default_post_fetch_count")]
    pub post_fetch_count: u32,
    pub feeds: Vec<Feed>,
}

fn default_post_fetch_count() -> u32 {
    50
}

#[derive(Deserialize, Debug)]
pub struct Feed {
    pub title: String,
    pub subreddit: String,
    #[serde(default = "default_color")]
    pub color: u32,
    pub trigger: Option<String>,
}

fn default_color() -> u32 {
    0xFF_FF_FF
}

impl Feed {
    pub fn get_trigger(self: &Self) -> &String {
        match &self.trigger {
            Some(trigger) => trigger,
            None => &self.title
        }
    }
}
