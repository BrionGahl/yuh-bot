use poise::serenity_prelude::RoleId;

/// Replaces `@RoleName` placeholders in `text` with each role's real Discord mention syntax
/// (`<@&ROLE_ID>`), so the role actually gets pinged instead of showing as plain text.
pub fn replace_role_mentions(text: &str, replacements: &[(&str, RoleId)]) -> String {
    let mut result = text.to_string();
    for (placeholder, role_id) in replacements {
        result = result.replace(placeholder, &format!("<@&{}>", role_id));
    }
    result
}
