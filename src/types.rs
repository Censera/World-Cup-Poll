use chrono::{DateTime, Utc};
use poise::serenity_prelude as sp;
use std::collections::HashMap as hm;
use std::sync::RwLock as rl;

#[derive(Clone, Default)]
pub struct UserPrediction {
    pub goals_for_team_a: Option<u8>,
    pub goals_for_team_b: Option<u8>,
}

pub struct FinalPrediction {
    pub user: sp::UserId,
    pub team_a: u8,
    pub team_b: u8,
}

pub struct PollInfo {
    pub start_time: DateTime<Utc>,
}

pub struct Data {
    pub drafts: rl<hm<sp::MessageId, hm<sp::UserId, UserPrediction>>>,
    pub finalized: rl<hm<sp::MessageId, Vec<FinalPrediction>>>,
    pub active_polls: rl<hm<sp::MessageId, PollInfo>>,
    pub user_points: rl<hm<sp::UserId, u32>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
