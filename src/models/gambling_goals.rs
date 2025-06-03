use chrono::{NaiveDate, Utc};
use serenity::all::UserId;
use sqlx::FromRow;

use crate::FormatNum;

use super::super::goals::GOAL_REGISTRY;

#[derive(FromRow)]
pub struct GamblingGoalsRow {
    pub user_id: i64,
    pub goal_id: String,
    pub day: NaiveDate,
    pub progress: i64,
    pub target: i64,
}

impl GamblingGoalsRow {
    pub fn goal_id(&self) -> &str {
        &self.goal_id
    }

    pub fn is_today(&self) -> bool {
        self.day == Utc::now().date_naive()
    }

    pub fn update_progress(&mut self, value: i64) {
        self.progress += value;
        self.progress = self.progress.min(self.target);
    }

    pub fn reset_progress(&mut self) {
        self.progress = 0
    }

    pub fn set_completed(&mut self) {
        self.progress = self.target
    }

    pub fn is_complete(&self) -> bool {
        self.progress == self.target
    }
}

impl GamblingGoalsRow {
    pub fn new(user_id: impl Into<UserId>, goal_id: impl Into<String>, target: i64) -> Self {
        let user_id = user_id.into();

        Self {
            user_id: user_id.get() as i64,
            goal_id: goal_id.into(),
            day: Utc::now().date_naive(),
            progress: 0,
            target,
        }
    }

    pub fn completed(&self) -> bool {
        self.progress == self.target
    }

    pub fn description(&self) -> String {
        let title = if let Some(goal) = GOAL_REGISTRY.get_definition(&self.goal_id) {
            (goal.description)(self.target)
        } else {
            self.goal_id.clone()
        };

        format!(
            "**{title}**\nProgress: `{}/{}`",
            self.progress.format(),
            self.target.format()
        )
    }
}
