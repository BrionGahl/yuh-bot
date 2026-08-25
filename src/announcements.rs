use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeZone};
use chrono_tz::{America::New_York, Tz};
use log::{error, info};
use poise::serenity_prelude::{ChannelId, CreateMessage, Error, Http};

const ANNOUNCEMENT_HOUR: u32 = 12;

/// Runs forever, posting `message` to `channel_id` every day at 12:00 PM Eastern. Uses
/// `America/New_York` rather than a fixed UTC offset so the announcement keeps landing at noon
/// local time across the EST/EDT switch, matching the convention used for the weekly raid notes
/// post.
pub async fn schedule_daily_announcement(http: Arc<Http>, channel_id: ChannelId, message: String) {
    loop {
        let now = chrono::Utc::now().with_timezone(&New_York);
        let target = next_noon(now);

        let sleep_duration = (target - now).to_std().unwrap_or(Duration::ZERO);
        info!("Next daily announcement scheduled for {}", target);
        tokio::time::sleep(sleep_duration).await;

        if let Err(e) = post_announcement(&http, channel_id, &message).await {
            error!("Failed to post daily announcement: {}", e);
        }
    }
}

/// Finds the next 12:00 PM strictly after `now`, in the same timezone as `now`.
fn next_noon(now: DateTime<Tz>) -> DateTime<Tz> {
    let mut date = now.date_naive();
    loop {
        let local = date.and_hms_opt(ANNOUNCEMENT_HOUR, 0, 0).expect("valid time");
        if let Some(candidate) = New_York.from_local_datetime(&local).single() {
            if candidate > now {
                return candidate;
            }
        }
        date = date.succ_opt().expect("date overflow while computing next noon");
    }
}

async fn post_announcement(http: &Http, channel_id: ChannelId, message: &str) -> Result<(), Error> {
    channel_id.send_message(http, CreateMessage::new().content(message)).await?;
    info!("Posted daily announcement");
    Ok(())
}
