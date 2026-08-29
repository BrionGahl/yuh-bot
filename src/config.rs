use std::collections::HashSet;
use std::env;

use cron::Schedule;
use poise::serenity_prelude::{ChannelId, RoleId, UserId};
use tracing_subscriber::filter::LevelFilter;

/// Cron expression used for the weekly raid notes post when `RAID_NOTES_CRON` is unset. Quartz-style
/// 7-field format (`sec min hour day-of-month month day-of-week year`): Fridays at 9:00 AM. Times are
/// evaluated in `America/New_York` (see `raid_notes.rs`), so this stays 9:00 AM local across the
/// EST/EDT switch.
const DEFAULT_RAID_NOTES_CRON: &str = "0 0 9 * * Fri *";

#[derive(Debug)]
pub struct Config {
    pub discord_token: String,
    pub bot_name: String,
    pub mod_role_id: RoleId,
    pub raider_role_id: RoleId,
    pub trial_role_id: RoleId,
    pub personal_officer_category_id: ChannelId,
    pub bart_token: String,
    pub wowutils_token: String,
    pub wowutils_group_id: String,
    pub clips_channel_ids: HashSet<ChannelId>,
    pub raid_notes_channel_id: ChannelId,
    /// Schedule for the weekly raid notes forum post, from `RAID_NOTES_CRON` (a quartz-style cron
    /// expression) or `DEFAULT_RAID_NOTES_CRON` when unset. Evaluated in `America/New_York`.
    pub raid_notes_cron: Schedule,
    pub announcement_channel_id: Option<ChannelId>,
    /// If set, when this user deletes one of their own messages the bot reposts it in the same
    /// channel — unless the audit log shows a moderator was the one who deleted it. Unset disables
    /// the feature entirely.
    pub replay_user_id: Option<UserId>,
    pub log_level: LevelFilter,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            discord_token: env::var("DISCORD_TOKEN")
                .expect("Missing `DISCORD_TOKEN` env variable."),
            bot_name: env::var("BOT_NAME")
                .unwrap_or("gulp-bot".to_string()),
            mod_role_id: RoleId::from(env::var("MOD_ROLE_ID")
                .expect("Missing `MOD_ROLE_ID` env variable.")
                .parse::<u64>()
                .expect("Failed to parse `MOD_ROLE_ID env variable")),
            raider_role_id: RoleId::from(env::var("RAIDER_ROLE_ID")
                .expect("Missing `RAIDER_ROLE_ID` env variable.")
                .parse::<u64>()
                .expect("Failed to parse `RAIDER_ROLE_ID env variable.")),
            trial_role_id: RoleId::from(env::var("TRIAL_ROLE_ID")
                .expect("Missing `TRIAL_ROLE_ID` env variable.")
                .parse::<u64>()
                .expect("Failed to parse `TRIAL_ROLE_ID env variable.")),
            personal_officer_category_id: ChannelId::from(env::var("PERSONAL_OFFICER_CATEGORY_ID")
                .expect("Missing `PERSONAL_OFFICER_CATEGORY_ID` env variable.")
                .parse::<u64>()
                .expect("Failed to parse `PERSONAL_OFFICER_CATEGORY_ID` env variable.")),
            bart_token: env::var("BART_TOKEN")
                .unwrap_or("".to_string()),
            wowutils_token: env::var("WOWUTILS_TOKEN")
                .expect("Missing `WOWUTILS_TOKEN` env variable."),
            wowutils_group_id: env::var("WOWUTILS_GROUP_ID")
                .expect("Missing `WOWUTILS_GROUP_ID` env variable."),
            clips_channel_ids: env::var("CLIPS_CHANNEL_IDS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| ChannelId::from(s.trim().parse::<u64>()
                    .expect("Failed to parse `CLIPS_CHANNEL_IDS` env variable, expected a comma-separated list of channel IDs.")))
                .collect(),
            raid_notes_channel_id: ChannelId::from(env::var("RAID_NOTES_CHANNEL_ID")
                .expect("Missing `RAID_NOTES_CHANNEL_ID` env variable.")
                .parse::<u64>()
                .expect("Failed to parse `RAID_NOTES_CHANNEL_ID` env variable.")),
            // Optional: unset (or empty) falls back to `DEFAULT_RAID_NOTES_CRON`. Quartz-style cron:
            // `sec min hour day-of-month month day-of-week [year]`, e.g. `0 0 9 * * Fri *`.
            raid_notes_cron: env::var("RAID_NOTES_CRON")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_RAID_NOTES_CRON.to_string())
                .parse::<Schedule>()
                .expect("Failed to parse `RAID_NOTES_CRON` env variable as a quartz-style cron expression (`sec min hour day-of-month month day-of-week [year]`)."),
            // Optional: if unset, the weekly Tuesday noon announcement is simply not scheduled
            // (see main.rs), so existing deploys don't break before this variable is configured.
            announcement_channel_id: env::var("ANNOUNCEMENT_CHANNEL_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|s| ChannelId::from(s.trim().parse::<u64>()
                    .expect("Failed to parse `ANNOUNCEMENT_CHANNEL_ID` env variable."))),
            // Optional: unset simply leaves the delete-and-replay feature off.
            replay_user_id: env::var("REPLAY_USER_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|s| UserId::from(s.trim().parse::<u64>()
                    .expect("Failed to parse `REPLAY_USER_ID` env variable."))),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "INFO".to_string())
                .parse::<LevelFilter>()
                .expect("Failed to parse `LOG_LEVEL` env variable. Valid values: TRACE, DEBUG, INFO, WARN, ERROR"),
        }
    }
}