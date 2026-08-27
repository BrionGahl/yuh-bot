use crate::config::Config;
use crate::message_replay::MessageReplay;

#[derive(Debug)]
pub struct Data {
    pub config: Config,
    pub http_client: reqwest::Client,
    /// Serializes the "does this member already have a personal officer channel, if not create
    /// one" sequence so that two GuildMemberUpdate events for the same member firing close
    /// together can't both pass the check before either has created a channel.
    pub personal_officer_channel_lock: tokio::sync::Mutex<()>,
    /// Buffers the configured replay user's recent messages so a deleted one can be reposted.
    pub message_replay: MessageReplay,
}

impl Data {
    pub fn new() -> Self {
        let config = Config::from_env();
        let http_client = reqwest::Client::new();

        Self {
            config,
            http_client,
            personal_officer_channel_lock: tokio::sync::Mutex::new(()),
            message_replay: MessageReplay::new(),
        }
    }
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
