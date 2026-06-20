use poise::serenity_prelude as sp;
use chrono::Utc;
use crate::types::{Data, Error, FinalPrediction};

pub async fn handle_interaction(
    ctx: &sp::Context,
    component: &sp::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let msg = component.message.id;
    let user = component.user.id;

    let (is_expired, start_timestamp) = {
        let active_polls = data.active_polls.read().unwrap();
        if let Some(info) = active_polls.get(&msg) {
            (Utc::now() > info.start_time, Some(info.start_time.timestamp()))
        } else {
            (false, None)
        }
    };

    if is_expired {
        let mut display = String::from("*Predictions are Locked.\nThe match has started.*\n\n");
        {
            let list = data.finalized.read().unwrap();
            if let Some(preds) = list.get(&msg) {
                for p in preds {
                    display.push_str(&format!("> <@{}> : **{} - {}**\n", p.user, p.team_a, p.team_b));
                }
            }
        }

        component
            .create_response(
                &ctx.http,
                sp::CreateInteractionResponse::UpdateMessage(
                    sp::CreateInteractionResponseMessage::new()
                        .embed(
                            sp::CreateEmbed::new()
                                .title(component.message.embeds[0].title.clone().unwrap_or_default())
                                .description(display)
                                .color(0xf4f800),
                        )
                        .components(vec![]),
                ),
            )
            .await?;
        return Ok(());
    }

    let has_voted = {
        let finalized = data.finalized.read().unwrap();
        if let Some(list) = finalized.get(&msg) {
            list.iter().any(|p| p.user == user)
        } else {
            false
        }
    };

    if has_voted {
        component
            .create_response(
                &ctx.http,
                sp::CreateInteractionResponse::Message(
                    sp::CreateInteractionResponseMessage::new()
                        .content("You already locked your score")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    match component.data.custom_id.as_str() {
        "score_team_a" => {
            let choice: u8;
            if let sp::ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
                choice = values[0].parse::<u8>().unwrap_or_else(|_| {
                  eprintln!("Failed to parse selection as u8: {}", values[0]);
                  0
                });
                data.drafts.write().unwrap().entry(msg).or_default().entry(user).or_default().goals_for_team_a = Some(choice);
            }
            component
                .create_response(&ctx.http, sp::CreateInteractionResponse::Acknowledge)
                .await?;
        }
        "score_team_b" => {
            let choice: u8;
            if let sp::ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
                choice = values[0].parse().unwrap_or(0);
                data.drafts.write().unwrap().entry(msg).or_default().entry(user).or_default().goals_for_team_b = Some(choice);
            }
            component
                .create_response(&ctx.http, sp::CreateInteractionResponse::Acknowledge)
                .await?;
        }
        "lock_prediction" => {
            let mut current_prediction = None;
            {
                let drafts = data.drafts.read().unwrap();
                if let Some(user_map) = drafts.get(&msg) {
                    if let Some(pred) = user_map.get(&user) {
                        if pred.goals_for_team_a.is_some() && pred.goals_for_team_b.is_some() {
                            current_prediction = Some(pred.clone());
                        }
                    }
                }
            }

            if let Some(pred) = current_prediction {
                data.finalized
                    .write()
                    .unwrap()
                    .entry(msg)
                    .or_default()
                    .push(FinalPrediction {
                        user,
                        team_a: pred.goals_for_team_a.unwrap(),
                        team_b: pred.goals_for_team_b.unwrap(),
                    });

                let mut display = String::new();
                if let Some(ts) = start_timestamp {
                  display.push_str(&format!("Select your score prediction\n\n**Match Starts: **<t:{}:R>\n\n", ts));
                }
                display.push_str("**Predictions:**\n");
                {
                    let list = data.finalized.read().unwrap();
                    if let Some(preds) = list.get(&msg) {
                        for p in preds {
                            display.push_str(&format!("> <@{}> : **{}** - **{}**\n", p.user, p.team_a, p.team_b));
                        }
                    }
                }

                component
                    .create_response(
                        &ctx.http,
                        sp::CreateInteractionResponse::UpdateMessage(
                            sp::CreateInteractionResponseMessage::new().embed(
                                sp::CreateEmbed::new()
                                    .title(component.message.embeds[0].title.clone().unwrap_or_default())
                                    .description(display)
                                    .color(0xff0069),
                            ),
                        ),
                    )
                    .await?;

                component
                    .create_followup(
                        &ctx.http,
                        sp::CreateInteractionResponseFollowup::new()
                            .content("Locked in, your prediction is added to the board.")
                            .ephemeral(true),
                    )
                    .await?;
            } else {
                component
                    .create_response(&ctx.http, sp::CreateInteractionResponse::Message(
                        sp::CreateInteractionResponseMessage::new()
                            .content("You must select a score for both teams")
                            .ephemeral(true),
                    ))
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}