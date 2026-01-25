// Rollback command - undo a release

use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

pub fn run(version: Option<String>) -> Result<()> {
    // Get version to rollback
    let version = match version {
        Some(v) => v.trim_start_matches('v').to_string(),
        None => get_latest_release()?,
    };

    let tag = format!("v{}", version);

    println!("{}", "=== Release Rollback ===".bold());
    println!();
    println!("This will rollback release: {}", tag.red());
    println!();
    println!("Actions:");
    println!("  1. Delete GitHub release {}", tag);
    println!("  2. Delete git tag {} (local and remote)", tag);
    println!();
    println!(
        "{}",
        "Note: Users who already downloaded this version will still have it.".dimmed()
    );
    println!();

    if !Confirm::new()
        .with_prompt("Are you sure you want to rollback this release?")
        .default(false)
        .interact()?
    {
        println!("Aborted.");
        return Ok(());
    }

    println!();

    // Delete GitHub release
    print!("Deleting GitHub release... ");
    match run_command("gh", &["release", "delete", &tag, "--yes"]) {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    // Delete remote tag
    print!("Deleting remote tag... ");
    match run_command("git", &["push", "--delete", "origin", &tag]) {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    // Delete local tag
    print!("Deleting local tag... ");
    match run_command("git", &["tag", "-d", &tag]) {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    println!();
    println!("{} Release {} rolled back", "✓".green(), tag);
    println!();
    println!("Next steps:");
    println!("  • Fix the issue that caused the rollback");
    println!("  • Bump version in Cargo.toml to a new version");
    println!("  • Push to main - a new release will be created automatically");

    Ok(())
}

/// Get the latest release version from GitHub
fn get_latest_release() -> Result<String> {
    let output = Command::new("gh")
        .args(["release", "list", "--limit", "1", "--json", "tagName"])
        .output()
        .with_context(|| "Failed to run gh command")?;

    if !output.status.success() {
        bail!("Failed to get releases from GitHub");
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let releases = json
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid response"))?;

    if releases.is_empty() {
        bail!("No releases found");
    }

    let tag = releases[0]["tagName"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No tag name found"))?;

    Ok(tag.trim_start_matches('v').to_string())
}

/// Run a command and return success/failure
fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run: {} {:?}", cmd, args))?;

    if !status.status.success() {
        bail!(
            "Command failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    Ok(())
}
