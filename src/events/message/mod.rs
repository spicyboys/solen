mod responders;

use poise::serenity_prelude::{Context, Message};

pub async fn handle_message(ctx: &Context, message: &Message) {
    for responder in responders::RESPONDERS.iter() {
        if let Err(e) = responder.respond(ctx, message).await {
            eprintln!("Responder error: {:?}", e);
        }
    }
}
