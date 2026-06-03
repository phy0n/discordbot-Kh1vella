use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::gateway::Ready,
};
use tracing::info;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("System initialized and connected as {}", ready.user.name);
    }
}
