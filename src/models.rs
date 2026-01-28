// Data models - Full implementation in Task #6

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session record
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub agent: Option<String>,
    pub summary: Option<String>,
    pub files_touched: Option<String>,        // JSON array
    pub status: String,                       // active, completed, abandoned
    pub full_context_shown: bool,
    pub structured_summary: Option<String>,   // JSON structured summary (v1.4)
}

/// Decision record
#[derive(Debug, Serialize, Deserialize)]
pub struct Decision {
    pub decision_id: i64,
    pub session_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub topic: String,
    pub decision: String,
    pub rationale: Option<String>,
    pub alternatives: Option<String>, // JSON array
    pub status: String,               // active, superseded, reversed
    pub superseded_by: Option<i64>,
}

/// Task record
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub task_id: i64,
    pub session_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub description: String,
    pub status: String,   // pending, in_progress, completed, blocked, cancelled
    pub priority: String, // low, normal, high, urgent
    pub blocked_by: Option<String>,
    pub parent_task_id: Option<i64>,
    pub notes: Option<String>,
}

/// Blocker record
#[derive(Debug, Serialize, Deserialize)]
pub struct Blocker {
    pub blocker_id: i64,
    pub session_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub description: String,
    pub status: String, // active, resolved, wont_fix
    pub resolution: Option<String>,
    pub related_task_id: Option<i64>,
}

/// Context note record
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextNote {
    pub note_id: i64,
    pub session_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub category: String, // goal, constraint, assumption, requirement, note
    pub title: String,
    pub content: String,
    pub status: String, // active, outdated, archived
}

/// Question record
#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub question_id: i64,
    pub session_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub question: String,
    pub context: Option<String>,
    pub answer: Option<String>,
    pub status: String, // open, answered, deferred
}

/// Milestone record
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Milestone {
    pub milestone_id: i64,
    pub created_at: DateTime<Utc>,
    pub target_date: Option<DateTime<Utc>>,
    pub achieved_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: Option<String>,
    pub status: String, // pending, achieved, missed, cancelled
}
