use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate};
use chrono_tz::America::New_York;
use cron::Schedule;
use log::{error, info};
use poise::serenity_prelude::{ChannelId, CreateForumPost, CreateMessage, Error, Http};

/// Runs forever, creating a new post in the raid notes forum channel on the schedule given by
/// `schedule` (the cron expression from `RAID_NOTES_CRON`). Fire times are computed in
/// `America/New_York` rather than a fixed UTC offset, so a schedule like "9:00 AM Friday" keeps
/// landing at 9:00 AM local time across the EST/EDT switch.
pub async fn schedule_weekly_posts(http: Arc<Http>, channel_id: ChannelId, schedule: Schedule) {
    loop {
        let now = chrono::Utc::now().with_timezone(&New_York);
        let Some(target) = schedule.after(&now).next() else {
            error!("RAID_NOTES_CRON schedule has no upcoming fire times; raid notes scheduler stopping.");
            return;
        };

        let sleep_duration = (target - now).to_std().unwrap_or(Duration::ZERO);
        info!("Next weekly raid notes post scheduled for {}", target);
        tokio::time::sleep(sleep_duration).await;

        if let Err(e) = create_raid_notes_post(&http, channel_id, target.date_naive()).await {
            error!("Failed to create weekly raid notes post: {}", e);
        }
    }
}

/// Creates the raid notes forum post for the week (Mon–Sun) containing `date`, with a placeholder
/// starter message. Returns the post title on success.
pub async fn create_raid_notes_post(
    http: &Http,
    channel_id: ChannelId,
    date: NaiveDate,
) -> Result<String, Error> {
    let monday = date - ChronoDuration::days(date.weekday().num_days_from_monday() as i64);
    let sunday = monday + ChronoDuration::days(6);
    let title = format!("Week of {} - {}", monday.format("%b %-d"), sunday.format("%b %-d"));

    let message = CreateMessage::new().content("Raid notes for this week go here.");
    channel_id.create_forum_post(http, CreateForumPost::new(title.clone(), message)).await?;

    info!("Created weekly raid notes post: {}", title);
    Ok(title)
}
