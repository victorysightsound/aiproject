// Release command - guide through the release process with reminders

use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

/// Release types for version bumping
const VERSION_TYPES: &[&str] = &["patch (x.x.X)", "minor (x.X.0)", "major (X.0.0)"];

pub fn run(check_only: bool) -> Result<()> {
    // Get current version from Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml")
        .with_context(|| "Could not read Cargo.toml - are you in the proj directory?")?;

    let current_version = extract_version(&cargo_toml)?;
    println!("Current version: {}", current_version.cyan());

    if check_only {
        println!("\nChecking release status...");
        check_release_status(&current_version)?;
        return Ok(());
    }

    // Interactive release process
    println!("\n{}", "=== Release Wizard ===".bold());

    // Step 1: Choose version bump
    println!("\n{}", "Step 1: Version Bump".bold());
    let selection = Select::new()
        .with_prompt("What type of release is this?")
        .items(VERSION_TYPES)
        .default(0)
        .interact()?;

    let new_version = bump_version(&current_version, selection)?;
    println!("New version will be: {}", new_version.green());

    if !Confirm::new()
        .with_prompt("Continue with this version?")
        .default(true)
        .interact()?
    {
        println!("Aborted.");
        return Ok(());
    }

    // Step 2: Update CHANGELOG
    println!("\n{}", "Step 2: Changelog".bold());
    println!(
        "{}",
        "Please update CHANGELOG.md with release notes before continuing.".yellow()
    );
    println!("Typical entries:");
    println!("  • New features added");
    println!("  • Bugs fixed");
    println!("  • Breaking changes (if any)");

    if !Confirm::new()
        .with_prompt("Have you updated CHANGELOG.md?")
        .default(false)
        .interact()?
    {
        println!("Please update CHANGELOG.md and run 'proj release' again.");
        return Ok(());
    }

    // Step 3: Update version in Cargo.toml
    println!("\n{}", "Step 3: Updating Cargo.toml".bold());
    update_cargo_version(&new_version)?;
    println!("{} Updated Cargo.toml to version {}", "✓".green(), new_version);

    // Step 4: Commit changes
    println!("\n{}", "Step 4: Committing changes".bold());
    let commit_msg = format!("Release v{}", new_version);

    run_command("git", &["add", "Cargo.toml", "Cargo.lock", "CHANGELOG.md"])?;
    run_command("git", &["commit", "-m", &commit_msg])?;
    println!("{} Committed: {}", "✓".green(), commit_msg);

    // Step 5: Create tag
    println!("\n{}", "Step 5: Creating tag".bold());
    let tag = format!("v{}", new_version);
    run_command("git", &["tag", &tag])?;
    println!("{} Created tag: {}", "✓".green(), tag);

    // Step 6: Push
    println!("\n{}", "Step 6: Pushing to GitHub".bold());
    if Confirm::new()
        .with_prompt("Push commits and tag to GitHub? (This triggers the release build)")
        .default(true)
        .interact()?
    {
        run_command("git", &["push"])?;
        run_command("git", &["push", "--tags"])?;
        println!("{} Pushed to GitHub", "✓".green());

        println!("\n{}", "=== Release Build Started ===".bold().green());
        println!("GitHub Actions is now building release binaries.");
        println!("Monitor progress: https://github.com/victorysightsound/aiproject/actions");

        // Show post-release reminders
        println!("\n{}", "=== Post-Release Checklist ===".bold().yellow());
        println!("After the GitHub Action completes (~5-10 min), run:");
        println!("  {}", "proj release --check".cyan());
        println!("\nThis will:");
        println!("  • Verify the release was created successfully");
        println!("  • Update Homebrew formula with new SHA256 hashes");
        println!("  • Show any remaining manual steps");
    } else {
        println!("\nTo complete the release later:");
        println!("  git push && git push --tags");
    }

    Ok(())
}

/// Check release status and update formulas
fn check_release_status(version: &str) -> Result<()> {
    let tag = format!("v{}", version);

    // Check if release exists on GitHub
    println!("\nChecking GitHub release...");
    let output = Command::new("gh")
        .args(["release", "view", &tag, "--json", "assets"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let json: serde_json::Value = serde_json::from_slice(&out.stdout)?;
            let assets = json["assets"].as_array();

            if let Some(assets) = assets {
                if assets.is_empty() {
                    println!("{} Release exists but has no assets yet.", "⏳".yellow());
                    println!("The build may still be in progress.");
                    println!("Check: https://github.com/victorysightsound/aiproject/actions");
                    return Ok(());
                }

                println!("{} Release {} found with {} assets", "✓".green(), tag, assets.len());

                // List assets
                for asset in assets {
                    if let Some(name) = asset["name"].as_str() {
                        println!("  • {}", name);
                    }
                }

                // Update Homebrew formula
                println!("\n{}", "Updating Homebrew formula...".bold());
                update_homebrew_formula(version)?;

                println!("\n{}", "=== Release Complete ===".bold().green());
                println!("\nUsers can now install/upgrade via:");
                println!("  • brew upgrade proj");
                println!("  • npm update -g create-aiproj");
                println!("  • Download from: https://github.com/victorysightsound/aiproject/releases/tag/{}", tag);
            }
        }
        Ok(_) => {
            println!("{} Release {} not found yet.", "⏳".yellow(), tag);
            println!("The GitHub Action may still be running.");
            println!("Check: https://github.com/victorysightsound/aiproject/actions");
        }
        Err(e) => {
            println!("{} Could not check release: {}", "⚠".yellow(), e);
            println!("Make sure 'gh' CLI is installed and authenticated.");
        }
    }

    Ok(())
}

/// Update Homebrew formula with new SHA256 hashes
fn update_homebrew_formula(version: &str) -> Result<()> {
    let formula_path = "packaging/homebrew/aiproject.rb";
    let tag = format!("v{}", version);

    // Download URLs for each platform
    let platforms = [
        ("aarch64-apple-darwin", "proj-aarch64-apple-darwin.tar.gz"),
        ("x86_64-apple-darwin", "proj-x86_64-apple-darwin.tar.gz"),
        ("aarch64-unknown-linux-gnu", "proj-aarch64-unknown-linux-gnu.tar.gz"),
        ("x86_64-unknown-linux-gnu", "proj-x86_64-unknown-linux-gnu.tar.gz"),
    ];

    let mut hashes: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for (platform, filename) in platforms {
        print!("  Fetching hash for {}... ", platform);

        // Download the file and compute hash
        let url = format!(
            "https://github.com/victorysightsound/aiproject/releases/download/{}/{}",
            tag, filename
        );

        match compute_remote_sha256(&url) {
            Ok(hash) => {
                println!("{}", "✓".green());
                hashes.insert(platform.to_string(), hash);
            }
            Err(e) => {
                println!("{} ({})", "✗".red(), e);
            }
        }
    }

    if hashes.len() < 4 {
        println!("{} Could not fetch all hashes. Formula not updated.", "⚠".yellow());
        return Ok(());
    }

    // Read and update the formula
    let mut formula = std::fs::read_to_string(formula_path)
        .with_context(|| format!("Could not read {}", formula_path))?;

    // Update version
    formula = update_formula_field(&formula, "version", version);

    // Update SHA256 hashes (this is a bit fragile, but works for our formula structure)
    for (platform, hash) in &hashes {
        formula = update_formula_sha256(&formula, platform, hash);
    }

    std::fs::write(formula_path, &formula)?;
    println!("{} Updated {}", "✓".green(), formula_path);

    // Commit the formula update
    run_command("git", &["add", formula_path])?;
    run_command("git", &["commit", "-m", &format!("Update Homebrew formula for v{}", version)])?;
    run_command("git", &["push"])?;
    println!("{} Committed and pushed formula update", "✓".green());

    Ok(())
}

/// Compute SHA256 of a remote file
fn compute_remote_sha256(url: &str) -> Result<String> {
    use sha2::{Sha256, Digest};

    let response = ureq::get(url)
        .call()
        .with_context(|| format!("Failed to download {}", url))?;

    let mut reader = response.into_reader();
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = std::io::Read::read(&mut reader, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Extract version from Cargo.toml content
fn extract_version(content: &str) -> Result<String> {
    for line in content.lines() {
        if line.starts_with("version") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    return Ok(line[start + 1..end].to_string());
                }
            }
        }
    }
    bail!("Could not find version in Cargo.toml")
}

/// Bump version based on type (0=patch, 1=minor, 2=major)
fn bump_version(current: &str, bump_type: usize) -> Result<String> {
    let parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() < 3 {
        bail!("Invalid version format: {}", current);
    }

    let (major, minor, patch) = (parts[0], parts[1], parts[2]);

    let new_version = match bump_type {
        0 => format!("{}.{}.{}", major, minor, patch + 1), // patch
        1 => format!("{}.{}.0", major, minor + 1),          // minor
        2 => format!("{}.0.0", major + 1),                  // major
        _ => bail!("Invalid bump type"),
    };

    Ok(new_version)
}

/// Update version in Cargo.toml
fn update_cargo_version(new_version: &str) -> Result<()> {
    let content = std::fs::read_to_string("Cargo.toml")?;
    let mut new_content = String::new();

    for line in content.lines() {
        if line.starts_with("version") && line.contains('"') {
            new_content.push_str(&format!("version = \"{}\"\n", new_version));
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') {
        new_content.pop();
    }

    std::fs::write("Cargo.toml", new_content)?;

    // Update Cargo.lock by running cargo check
    let _ = Command::new("cargo")
        .args(["check", "--quiet"])
        .status();

    Ok(())
}

/// Update a field in the Homebrew formula
fn update_formula_field(formula: &str, field: &str, value: &str) -> String {
    let mut result = String::new();
    for line in formula.lines() {
        if line.trim().starts_with(field) && line.contains('"') {
            // Find the pattern and replace
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    let prefix = &line[..start + 1];
                    let suffix = &line[end..];
                    result.push_str(&format!("{}{}{}\n", prefix, value, suffix));
                    continue;
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Update SHA256 hash in formula for a specific platform
fn update_formula_sha256(formula: &str, platform: &str, hash: &str) -> String {
    let lines: Vec<&str> = formula.lines().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        result.push_str(line);
        result.push('\n');

        // Look for URL line containing our platform
        if line.contains("url") && line.contains(platform) {
            // Next non-empty line should be sha256
            i += 1;
            while i < lines.len() {
                let next_line = lines[i];
                if next_line.trim().starts_with("sha256") {
                    // Replace the hash
                    if let Some(start) = next_line.find('"') {
                        let prefix = &next_line[..start + 1];
                        result.push_str(&format!("{}{}\"\n", prefix, hash));
                        i += 1;
                        continue;
                    }
                }
                result.push_str(next_line);
                result.push('\n');
                i += 1;
                if !next_line.trim().is_empty() {
                    break;
                }
            }
            continue;
        }
        i += 1;
    }

    result
}

/// Run a command and check for success
fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run: {} {:?}", cmd, args))?;

    if !status.success() {
        bail!("Command failed: {} {:?}", cmd, args);
    }

    Ok(())
}
