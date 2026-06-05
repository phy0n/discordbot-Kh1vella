use crate::types::{Context, Error};
use serenity::all::UserId;

pub async fn is_staff(ctx: Context<'_>) -> Result<bool, Error> {
    let pool = &ctx.data().db;
    
    let user_id = ctx.author().id.to_string();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM khivella_access WHERE discord_id = $1")
        .bind(&user_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
        
    if count.0 > 0 {
        Ok(true)
    } else {
        crate::utils::embeds::send_embed(ctx, "Access Denied", "⚠️ **Clearance Level Insufficient.**\nYour Discord ID is not registered in the Staff Operations Center. Please contact a root administrator to request system access.", 0xED4245).await?;
        Ok(false)
    }
}
