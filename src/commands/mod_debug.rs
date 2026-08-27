use crate::announcements::{post_announcement, rendered_announcement};
use crate::types::bot::{Context, Error};

/// Immediately posts the configured weekly announcement, for testing without waiting for Tuesday
/// noon.
#[poise::command(
    prefix_command,
    slash_command,
    ephemeral,
    category = "Debug",
    check = "crate::checks::check_is_moderator",
)]
pub async fn test_announcement(ctx: Context<'_>) -> Result<(), Error> {
    let Some(channel_id) = ctx.data().config.announcement_channel_id else {
        ctx.say("`ANNOUNCEMENT_CHANNEL_ID` is not configured; there's no channel to post to.").await?;
        return Ok(());
    };

    let message = rendered_announcement(ctx.data().config.raider_role_id, ctx.data().config.trial_role_id);

    post_announcement(ctx.serenity_context().http.as_ref(), channel_id, &message).await?;

    ctx.say(format!("Posted the announcement to <#{}>.", channel_id)).await?;
    Ok(())
}

/// Post a sample delete-and-replay embed here, to preview how a reposted message looks.
///
/// Uses you as the stand-in author. Only exercises the embed rendering — it doesn't touch the
/// audit log check or delete anything.
#[poise::command(
    prefix_command,
    slash_command,
    ephemeral,
    category = "Debug",
    check = "crate::checks::check_is_moderator",
)]
pub async fn test_replay(
    ctx: Context<'_>,
    #[description = "Text to show in the replayed embed"] message: Option<String>,
) -> Result<(), Error> {
    let content = message.unwrap_or_else(|| "This is a sample deleted message.".to_string());

    crate::message_replay::render_test_replay(
        ctx.serenity_context(),
        ctx.channel_id(),
        ctx.author(),
        &content,
    )
    .await;

    ctx.say("Posted a sample replay embed.").await?;
    Ok(())
}
