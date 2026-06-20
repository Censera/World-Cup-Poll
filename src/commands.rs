use crate::types::{Context, Error, PollInfo};
use chrono::DateTime;
use poise::serenity_prelude as sp;

fn goal_options() -> Vec<sp::CreateSelectMenuOption> {
    (0..=10) // limit is 10 goals and it is more then enough (most of the times)
        .map(|i| {
            sp::CreateSelectMenuOption::new(
                format!("{} {}", i, if i == 1 { "Goal" } else { "Goals" }),
                i.to_string(),
            )
        })
        .collect()
}
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_MESSAGES | MANAGE_EVENTS"
)]
pub async fn poll_match(
    ctx: Context<'_>,
    #[description = "[flag] team A name"] team_a: String,
    #[description = "[flag] team B name"] team_b: String,
    #[description = "Kickoff time (YYYY/MM/DD H:M)"] kickoff: String,
) -> Result<(), Error> {
    let naive_datetime = chrono::NaiveDateTime::parse_from_str(&kickoff, "%Y/%m/%d %H:%M")
        .map_err(|_| "Please use YYYY/MM/DD HH:MM e.g. (2026/6/20 18:00)")?;
    let start_time =
        DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_datetime, chrono::Utc);

    let match_title = format!("**{}** vs. **{}**", team_a, team_b);

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .embed(
                    sp::CreateEmbed::default()
                        .title(format!("{}", match_title))
                        .description(format!(
                            "Select your score prediction\n\n**Match Starts:** <t:{}:R>\n\n*No predictions yet.*", // <t::R> formats the time ig
                            start_time.timestamp()
                        ))
                        .color(0xFF0000),
                )
                .components(vec![
                    sp::CreateActionRow::SelectMenu(
                        sp::CreateSelectMenu::new(
                            "score_team_a",
                            sp::CreateSelectMenuKind::String {
                                options: goal_options(),
                            },
                        )
                        .placeholder(format!("{} Score", team_a)),
                    ),
                    sp::CreateActionRow::SelectMenu(
                        sp::CreateSelectMenu::new(
                            "score_team_b",
                            sp::CreateSelectMenuKind::String {
                                options: goal_options(),
                            },
                        )
                        .placeholder(format!("{} Score", team_b)),
                    ),
                    sp::CreateActionRow::Buttons(vec![
                        sp::CreateButton::new("lock_prediction")
                            .label("Lock In Score")
                            .style(sp::ButtonStyle::Success),
                    ]),
                ]),
        )
        .await?;

    let msg_id = reply.message().await?.id;
    ctx.data()
        .active_polls
        .write()
        .unwrap()
        .insert(msg_id, PollInfo { start_time });

    Ok(())
}

#[poise::command(slash_command)]
pub async fn poll_about(ctx: Context<'_>) -> Result<(), Error> {
    // about or help
    ctx.send(
        poise::CreateReply::default().embed(
            sp::CreateEmbed::new()
                .title("World Cup Polls")
                .description(
                    "A bot for tracking live World Cup score predictions.\n\nWhen a match poll is posted, use the dropdown menus to lock in your score before kickoff"
                )
                .footer(sp::CreateEmbedFooter::new("By Censera"))
                .color(0xff0069)
        )
        .ephemeral(true),
    )
    .await?;
    Ok(())
}
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_MESSAGES | MANAGE_EVENTS"
)]
pub async fn set_match_score(
    ctx: Context<'_>,
    #[description = "The Message ID of the match poll"] message_id_str: String,
    #[description = "Actual score for Team A"] score_a: u8,
    #[description = "Actual score for Team B"] score_b: u8,
) -> Result<(), Error> {
    let msg_id: sp::MessageId = message_id_str
        .parse::<u64>()
        .map_err(|_| "invalid Message ID format.")?
        .into();

    ctx.defer().await?;

    {
        let mut active = ctx.data().active_polls.write().unwrap();
        active.remove(&msg_id);
    }

    let predictions = {
        let mut finalized = ctx.data().finalized.write().unwrap();
        finalized.remove(&msg_id).unwrap_or_default()
    };

    if predictions.is_empty() {
        ctx.say("No predictions were locked in for this match.")
            .await?;
        return Ok(());
    }

    let actual_diff = score_a as i8 - score_b as i8;
    let mut exact_winners = Vec::new();
    let mut outcome_winners = Vec::new();

    {
        let mut points_lock = ctx.data().user_points.write().unwrap();

        for pred in &predictions {
            let pred_diff = pred.team_a as i8 - pred.team_b as i8;

            let pts = if score_a == pred.team_a && score_b == pred.team_b {
                exact_winners.push(format!(
                    "<@{}> ({}-{})",
                    pred.user, pred.team_a, pred.team_b
                ));
                3
            } else if (actual_diff > 0 && pred_diff > 0)
                || (actual_diff < 0 && pred_diff < 0)
                || (actual_diff == 0 && pred_diff == 0)
            {
                outcome_winners.push(format!(
                    "<@{}> ({}-{})",
                    pred.user, pred.team_a, pred.team_b
                ));
                1
            } else {
                0
            };

            if pts > 0 {
                *points_lock.entry(pred.user).or_default() += pts;
            }
        }
    }

    let mut results_msg = format!(
        "### Match ended. Final Score: {} - {}\n\n",
        score_a, score_b
    );

    results_msg.push_str("Exact Score Winners:\n");
    if exact_winners.is_empty() {
        results_msg.push_str("> *None* <:AHAHAHA:1517999261743054948>\n");
    } else {
        results_msg.push_str(&format!("> {}\n", exact_winners.join(", ")));
    }

    results_msg.push_str("\nCorrect Outcome Winners:\n");
    if outcome_winners.is_empty() {
        results_msg.push_str("> *None* <:AHAHAHA:1517999261743054948>\n");
    } else {
        results_msg.push_str(&format!("> {}\n", outcome_winners.join(", ")));
    }

    ctx.say(results_msg).await?;

    let _ = ctx
        .channel_id()
        .edit_message(
            &ctx.http(),
            msg_id,
            sp::EditMessage::new()
                .embed(
                    sp::CreateEmbed::new()
                        .title("Ended")
                        .description(format!(
                            "This poll is closed.\nFinal Score: **{}** - **{}**",
                            score_a, score_b
                        ))
                        .color(0xff0069),
                )
                .components(vec![]),
        )
        .await;

    Ok(())
}
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_MESSAGES | MANAGE_EVENTS"
)]
pub async fn poll_help(ctx: Context<'_>) -> Result<(), Error> {
    //  help
    ctx.send(
        poise::CreateReply::default().embed(
            sp::CreateEmbed::new()
                .title("World Cup Polls")
                .description(
                    "**Recommaneded Format**\nFor teams:\n- [Flag Team] Flag (e.g. :flag_ma: Morocco)\nFor the time:\n- YEAR/MONTH/DAY HOUR:MINUTE (e.g. 2026/6/22 14:30)"
                )
                .footer(sp::CreateEmbedFooter::new("Contact @Censera for any issues"))
                .color(0xff0069)
        )
        .ephemeral(true),
    )
    .await?;
    Ok(())
}
