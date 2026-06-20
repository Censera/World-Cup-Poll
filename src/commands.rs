use poise::serenity_prelude as sp;
use chrono::DateTime;
use crate::types::{Context, Error, PollInfo};

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

#[poise::command(slash_command)]
pub async fn score_poll(
    ctx: Context<'_>,
    #[description = "[flag] team A name"] team_a: String,
    #[description = "[flag] team B name"] team_b: String,
    #[description = "Kickoff time (YYYY/MM/DD H:M)"] kickoff: String,
) -> Result<(), Error> {
    let naive_datetime = chrono::NaiveDateTime::parse_from_str(&kickoff, "%Y/%m/%d %H:%M")
        .map_err(|_| "Please use YYYY/MM/DD HH:MM e.g. (2026/6/20 18:00)")?;
    let start_time = DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_datetime, chrono::Utc);

    let match_title = format!("**{}** vs. **{}**", team_a, team_b);

    let reply = ctx.send(
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
                        sp::CreateSelectMenuKind::String { options: goal_options() },
                    ).placeholder(format!("{} Score", team_a)),
                ),
                sp::CreateActionRow::SelectMenu(
                    sp::CreateSelectMenu::new(
                        "score_team_b",
                        sp::CreateSelectMenuKind::String { options: goal_options() },
                    ).placeholder(format!("{} Score", team_b)),
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
    ctx.data().active_polls.write().unwrap().insert(
        msg_id,
        PollInfo { start_time },
    );

    Ok(())
}

#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> { // about or help 
    ctx.send(
        poise::CreateReply::default().embed(
            sp::CreateEmbed::new()
                .title("World Cup Polls")
                .description(
                    ".."
                )
                .color(0x00ffac00)
        )
        .ephemeral(true),
    )
    .await?;
    Ok(())
}