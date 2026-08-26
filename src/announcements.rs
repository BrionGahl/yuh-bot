use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Datelike, TimeZone, Weekday};
use chrono_tz::{America::New_York, Tz};
use log::{error, info};
use poise::serenity_prelude::{ChannelId, CreateMessage, Error, Http, RoleId};

const ANNOUNCEMENT_HOUR: u32 = 12;
const ANNOUNCEMENT_WEEKDAY: Weekday = Weekday::Tue;

/// Runs forever, posting `message` to `channel_id` every Tuesday at 12:00 PM Eastern. Uses
/// `America/New_York` rather than a fixed UTC offset so the announcement keeps landing at noon
/// local time across the EST/EDT switch, matching the convention used for the weekly raid notes
/// post.
///
/// `message` is a template straight from the `ANNOUNCEMENT_MESSAGE` env var: it may contain
/// literal `\n` sequences (the deploy pipeline can't carry real newlines through Cloud Run's
/// `env_vars` KV list without them being mistaken for separate variables) and `@Raider`/`@Trial`
/// placeholders, both expanded once up front by `render_message`.
pub async fn schedule_weekly_announcement(
    http: Arc<Http>,
    channel_id: ChannelId,
    message: String,
    raider_role_id: RoleId,
    trial_role_id: RoleId,
) {
    let message = render_message(&message, raider_role_id, trial_role_id);

    loop {
        let now = chrono::Utc::now().with_timezone(&New_York);
        let target = next_tuesday_noon(now);

        let sleep_duration = (target - now).to_std().unwrap_or(Duration::ZERO);
        info!("Next weekly announcement scheduled for {}", target);
        tokio::time::sleep(sleep_duration).await;

        if let Err(e) = post_announcement(&http, channel_id, &message).await {
            error!("Failed to post weekly announcement: {}", e);
        }
    }
}

/// Expands escaped newlines and role-name placeholders in the raw `ANNOUNCEMENT_MESSAGE` template
/// into what should actually be sent to Discord.
pub fn render_message(template: &str, raider_role_id: RoleId, trial_role_id: RoleId) -> String {
    template
        .replace("\\n", "\n")
        .replace("@Raider", &format!("<@&{}>", raider_role_id))
        .replace("@Trial", &format!("<@&{}>", trial_role_id))
}

/// Finds the next Tuesday 12:00 PM strictly after `now`, in the same timezone as `now`.
fn next_tuesday_noon(now: DateTime<Tz>) -> DateTime<Tz> {
    let mut date = now.date_naive();
    loop {
        if date.weekday() == ANNOUNCEMENT_WEEKDAY {
            let local = date.and_hms_opt(ANNOUNCEMENT_HOUR, 0, 0).expect("valid time");
            if let Some(candidate) = New_York.from_local_datetime(&local).single() {
                if candidate > now {
                    return candidate;
                }
            }
        }
        date = date.succ_opt().expect("date overflow while computing next Tuesday noon");
    }
}

pub async fn post_announcement(http: &Http, channel_id: ChannelId, message: &str) -> Result<(), Error> {
    channel_id.send_message(http, CreateMessage::new().content(message)).await?;
    info!("Posted weekly announcement");
    Ok(())
}
