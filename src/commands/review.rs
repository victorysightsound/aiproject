// Review command - cleanup pass for missed logging
//
// Shows session activity alongside git commits to help identify
// decisions, tasks, or context that wasn't logged during the session.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::database::open_database;
use crate::git;
use crate::paths::get_tracking_db_path;
use crate::session::get_active_session;

pub fn run() -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    // Get active session
    let session = match get_active_session(&conn)? {
        Some(s) => s,
        None => {
            println!("{} No active session to review.", "○".white());
            println!("Run 'proj status' to start a session first.");
            return Ok(());
        }
    };

    let session_start = session.started_at.format("%Y-%m-%d %H:%M:%S").to_string();

    println!("{}", "Session Review".bold());
    println!("{}", "═".repeat(50));
    println!();

    // Count logged items
    let decision_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM decisions WHERE session_id = ?",
            [session.session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let task_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE session_id = ?",
            [session.session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let note_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM context_notes WHERE session_id = ?",
            [session.session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let blocker_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM blockers WHERE session_id = ?",
            [session.session_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Get git commits since session start
    let commits = git::get_commits_since(&conn, &session_start)?;
    let commit_count = commits.len();

    // Show what's been logged
    println!("{}", "Logged This Session:".bold());
    println!(
        "  Decisions: {}  Tasks: {}  Notes: {}  Blockers: {}",
        if decision_count > 0 {
            decision_count.to_string().green()
        } else {
            "0".yellow()
        },
        if task_count > 0 {
            task_count.to_string().green()
        } else {
            "0".white()
        },
        if note_count > 0 {
            note_count.to_string().green()
        } else {
            "0".white()
        },
        if blocker_count > 0 {
            blocker_count.to_string().green()
        } else {
            "0".white()
        },
    );
    println!();

    // Show git commits
    println!(
        "{} ({} commits)",
        "Git Commits This Session:".bold(),
        commit_count
    );
    if commits.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for commit in commits.iter().take(15) {
            println!(
                "  {} {}",
                commit.short_hash.cyan(),
                truncate(&commit.message, 60)
            );
        }
        if commits.len() > 15 {
            println!("  {} more...", commits.len() - 15);
        }
    }
    println!();

    // Show logged decisions (so LLM can see what's already captured)
    if decision_count > 0 {
        println!("{}", "Decisions Already Logged:".bold());
        let mut stmt = conn.prepare(
            "SELECT topic, decision FROM decisions WHERE session_id = ? ORDER BY created_at",
        )?;
        let decisions: Vec<(String, String)> = stmt
            .query_map([session.session_id], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        for (topic, decision) in decisions {
            println!("  • {}: {}", topic.bold(), truncate(&decision, 50));
        }
        println!();
    }

    // Analysis and recommendations
    println!("{}", "Review Checklist:".bold());

    if commit_count > 0 && decision_count == 0 {
        println!(
            "  {} {} commits but no decisions logged.",
            "⚠".yellow(),
            commit_count
        );
        println!("    Review commits above - were any architectural/design choices made?");
        println!(
            "    Log with: {}",
            "proj log decision \"topic\" \"decision\" \"rationale\"".cyan()
        );
    } else if commit_count as i64 > decision_count * 3 {
        println!(
            "  {} Many commits ({}) vs few decisions ({}).",
            "💡".yellow(),
            commit_count,
            decision_count
        );
        println!("    Consider if any decisions are missing.");
    } else {
        println!("  {} Decision coverage looks reasonable.", "✓".green());
    }

    if commit_count > 3 && task_count == 0 {
        println!("  {} Several commits but no tasks logged.", "💡".yellow());
        println!("    Any follow-up work or TODOs identified?");
        println!(
            "    Log with: {}",
            "proj task add \"description\" --priority normal".cyan()
        );
    }

    println!();
    println!(
        "{}",
        "Review the commits and log any missed decisions/tasks.".dimmed()
    );
    println!(
        "{}",
        "When done, run 'proj session end \"summary\"' to close the session.".dimmed()
    );

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
