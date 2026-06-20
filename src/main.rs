mod commands;
mod interactions;
mod types;

use poise::serenity_prelude as sp;
use std::collections::HashMap as hm;
use std::sync::RwLock as rl;
use types::Data;

#[tokio::main]
async fn main() {
    let token = std::env::var("D_TOKEN").expect("missing D_TOKEN in env");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::poll_match(),
                commands::poll_about(),
                commands::poll_help(),
                commands::set_match_score(),
            ],
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    match event {
                        sp::FullEvent::InteractionCreate { interaction } => {
                            if let sp::Interaction::Component(component) = interaction {
                                if let Err(e) =
                                    interactions::handle_interaction(ctx, component, data).await
                                {
                                    eprintln!("interaction error: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                log("bot, commands are ready and registered");

                Ok(Data {
                    drafts: rl::new(hm::new()),
                    finalized: rl::new(hm::new()),
                    active_polls: rl::new(hm::new()),
                    user_points: rl::new(hm::new()),
                })
            })
        })
        .build();

    let mut client = sp::ClientBuilder::new(token, sp::GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .expect("failed to create a client");

    client.start().await.expect("client Error");
}

fn log(msg: &str) {
    println!(
        "[{}] {}",
        chrono::Local::now().format("%Y/%m/%d %H:%M:%S"),
        msg
    );
}
