use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Datelike, TimeZone, Weekday};
use chrono_tz::{America::New_York, Tz};
use log::{error, info};
use poise::serenity_prelude::{ChannelId, CreateMessage, Error, Http, RoleId};

use crate::mentions::replace_role_mentions;

const ANNOUNCEMENT_HOUR: u32 = 12;
const ANNOUNCEMENT_WEEKDAY: Weekday = Weekday::Tue;

/// Template for the weekly announcement, with `@Raider`/`@Trial` placeholders expanded into real
/// role mentions by `rendered_announcement`. Baked into the binary at compile time (rather than
/// read from an env var) so it can hold arbitrary commas/newlines/formatting without fighting the
/// deploy pipeline's KEY=VALUE env var parsing.
const ANNOUNCEMENT_TEMPLATE: &str = include_str!("../resources/announcement_message.txt");

/// Runs forever, posting the announcement to `channel_id` every Tuesday at 12:00 PM Eastern. Uses
/// `America/New_York` rather than a fixed UTC offset so the announcement keeps landing at noon
/// local time across the EST/EDT switch, matching the convention used for the weekly raid notes
/// post.
pub async fn schedule_weekly_announcement(
    http: Arc<Http>,
    channel_id: ChannelId,
    raider_role_id: RoleId,
    trial_role_id: RoleId,
) {
    let message = rendered_announcement(raider_role_id, trial_role_id);

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

/// Expands the `@Raider`/`@Trial` placeholders in `ANNOUNCEMENT_TEMPLATE` into real role mentions.
pub fn rendered_announcement(raider_role_id: RoleId, trial_role_id: RoleId) -> String {
    replace_role_mentions(ANNOUNCEMENT_TEMPLATE, &[
        ("@Raider", raider_role_id),
        ("@Trial", trial_role_id),
    ])
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
