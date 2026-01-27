// Update check - check for new versions and notify users

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::paths::{ensure_dir, get_global_dir, get_pending_update_dir};

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/victorysightsound/aiproject/releases/latest";
const GITHUB_REPO: &str = "victorysightsound/aiproject";
const CHECK_INTERVAL_HOURS: u64 = 24;

#[derive(Debug, Serialize, Deserialize)]
struct VersionCache {
    latest_version: String,
    checked_at: u64, // Unix timestamp
    download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

/// Get the path to the version cache file
fn get_cache_path() -> Result<PathBuf> {
    let global_dir = get_global_dir()?;
    Ok(global_dir.join("version_cache.json"))
}

/// Check if we should perform a version check (based on cache age)
fn should_check(cache_path: &PathBuf) -> bool {
    if !cache_path.exists() {
        return true;
    }

    if let Ok(metadata) = fs::metadata(cache_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                return elapsed > Duration::from_secs(CHECK_INTERVAL_HOURS * 3600);
            }
        }
    }
    true
}

/// Read cached version info
fn read_cache(cache_path: &PathBuf) -> Option<VersionCache> {
    if let Ok(content) = fs::read_to_string(cache_path) {
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

/// Write version info to cache
fn write_cache(cache_path: &PathBuf, cache: &VersionCache) -> Result<()> {
    let content = serde_json::to_string_pretty(cache)?;
    fs::write(cache_path, content)?;
    Ok(())
}

/// Fetch latest version from GitHub API
fn fetch_latest_version() -> Option<(String, String)> {
    // Use a short timeout to avoid slowing down CLI
    let client = match ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .build()
        .get(GITHUB_API_URL)
        .set("User-Agent", "proj-cli")
        .call()
    {
        Ok(response) => response,
        Err(_) => return None,
    };

    let release: GitHubRelease = match client.into_json() {
        Ok(r) => r,
        Err(_) => return None,
    };

    // Remove 'v' prefix if present
    let version = release.tag_name.trim_start_matches('v').to_string();
    Some((version, release.html_url))
}

/// Parse version string into comparable parts
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some((major, minor, patch))
    } else if parts.len() == 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor, 0))
    } else {
        None
    }
}

/// Compare versions, returns true if latest > current
fn is_newer(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some((c_maj, c_min, c_pat)), Some((l_maj, l_min, l_pat))) => {
            (l_maj, l_min, l_pat) > (c_maj, c_min, c_pat)
        }
        _ => false,
    }
}

/// Get the platform target string for binary downloads
fn get_platform_target() -> Option<&'static str> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Some("aarch64-apple-darwin");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return Some("x86_64-apple-darwin");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Some("x86_64-unknown-linux-gnu");
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return Some("aarch64-unknown-linux-gnu");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Some("x86_64-pc-windows-msvc");
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    return None;
}

/// Download update binary to staging directory
fn download_update(version: &str) -> Result<()> {
    let target = get_platform_target().ok_or_else(|| anyhow!("Unsupported platform"))?;

    #[cfg(target_os = "windows")]
    let ext = "zip";
    #[cfg(not(target_os = "windows"))]
    let ext = "tar.gz";

    let url = format!(
        "https://github.com/{}/releases/download/v{}/proj-{}.{}",
        GITHUB_REPO, version, target, ext
    );

    // Download with short timeout
    let response = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .build()
        .get(&url)
        .set("User-Agent", "proj-cli")
        .call()
        .map_err(|e| anyhow!("Download failed: {}", e))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;

    // Extract to staging
    let staging = get_pending_update_dir()?;

    // Clean up any existing staging directory
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    ensure_dir(&staging)?;

    #[cfg(not(target_os = "windows"))]
    {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let decoder = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(decoder);
        archive
            .unpack(&staging)
            .map_err(|e| anyhow!("Failed to extract archive: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows zip handling would go here
        // For now, return an error indicating manual update needed
        return Err(anyhow!(
            "Auto-update on Windows not yet supported, please update manually"
        ));
    }

    // Write version file
    fs::write(staging.join("version"), version)?;

    Ok(())
}

/// Spawn background thread to download update
pub fn download_update_background(version: String) {
    std::thread::spawn(move || {
        if let Err(_e) = download_update(&version) {
            // Silent fail - will retry on next check
            // Could log to file for debugging if needed
        }
    });
}

/// Check if an update is already staged
fn is_update_staged(version: &str) -> bool {
    if let Ok(staging) = get_pending_update_dir() {
        if let Ok(staged_version) = fs::read_to_string(staging.join("version")) {
            return staged_version.trim() == version;
        }
    }
    false
}

/// Check for updates and print notification if available
/// Returns true if an update is available
pub fn check_and_notify() -> bool {
    let cache_path = match get_cache_path() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version: String;
    let download_url: Option<String>;

    if should_check(&cache_path) {
        // Fetch from GitHub
        if let Some((version, url)) = fetch_latest_version() {
            latest_version = version.clone();
            download_url = Some(url);

            // Cache the result
            let cache = VersionCache {
                latest_version: version,
                checked_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                download_url: download_url.clone(),
            };
            let _ = write_cache(&cache_path, &cache);
        } else {
            // Network error, use cache if available
            if let Some(cache) = read_cache(&cache_path) {
                latest_version = cache.latest_version;
                download_url = cache.download_url;
            } else {
                return false;
            }
        }
    } else {
        // Use cached version
        if let Some(cache) = read_cache(&cache_path) {
            latest_version = cache.latest_version;
            download_url = cache.download_url;
        } else {
            return false;
        }
    }

    // Check if update is available
    if is_newer(current_version, &latest_version) {
        // Start background download if not already staged and platform supported
        if !is_update_staged(&latest_version) && get_platform_target().is_some() {
            download_update_background(latest_version.clone());
        }

        println!();
        println!(
            "{} Update available: {} → {}",
            "⬆".cyan(),
            current_version.yellow(),
            latest_version.green()
        );
        println!("  Install: {}", "brew upgrade proj".cyan());
        println!("       or: {}", "npm update -g create-aiproj".cyan());
        println!(
            "       or: {}",
            "cargo install --path ~/projects/global/tools/proj".cyan()
        );
        if let Some(url) = download_url {
            println!("  Release: {}", url.dimmed());
        }
        println!();
        return true;
    }

    false
}

/// Check if project schema needs upgrading and notify user (once per session)
/// This compares the project's schema_version with the current SCHEMA_VERSION
pub fn check_schema_upgrade(project_schema: &str, session_full_context_shown: bool) {
    // Only show on first context of session (avoid spamming)
    if session_full_context_shown {
        return;
    }

    let current_schema = crate::SCHEMA_VERSION;

    // Compare versions
    if is_newer(project_schema, current_schema) {
        println!(
            "{} Schema upgrade available: v{} → v{}",
            "⬆".cyan(),
            project_schema.yellow(),
            current_schema.green()
        );
        println!("  Run: {}", "proj upgrade".cyan());
        println!();
    }
}

/// Force a version check (for manual `proj update` command)
pub fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version.cyan());
    println!("Checking for updates...");

    match fetch_latest_version() {
        Some((latest, url)) => {
            if is_newer(current_version, &latest) {
                println!();
                println!("{} New version available: {}", "⬆".green(), latest.green());
                println!();
                println!("Install options:");
                println!("  • Homebrew:  {}", "brew upgrade proj".cyan());
                println!("  • npm:       {}", "npm update -g create-aiproj".cyan());
                println!(
                    "  • From source: {}",
                    "cargo install --path ~/projects/global/tools/proj".cyan()
                );
                println!();
                println!("Release notes: {}", url);
            } else {
                println!("{} You're on the latest version!", "✓".green());
            }

            // Update cache
            if let Ok(cache_path) = get_cache_path() {
                let cache = VersionCache {
                    latest_version: latest,
                    checked_at: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                    download_url: Some(url),
                };
                let _ = write_cache(&cache_path, &cache);
            }
        }
        None => {
            println!(
                "{} Could not check for updates (network error)",
                "⚠".yellow()
            );
        }
    }

    Ok(())
}
