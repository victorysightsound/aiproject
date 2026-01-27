// Auto-update - check for and apply pending updates at startup

use std::fs;
use std::process::Command;

use anyhow::Result;
use colored::Colorize;

use crate::paths::get_pending_update_dir;

/// Check for and apply a pending update at startup
/// Returns Ok(true) if an update was applied (process will exit and re-exec)
/// Returns Ok(false) if no pending update exists
/// Returns Err if something went wrong (caller should continue normally)
pub fn check_and_apply_pending() -> Result<bool> {
    let staging = get_pending_update_dir()?;
    let pending_binary = staging.join("proj");
    let version_file = staging.join("version");

    // No pending update
    if !pending_binary.exists() {
        return Ok(false);
    }

    // Get current binary path
    let current_exe = std::env::current_exe()?;

    // Check if we can write to the current binary location
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        // Check if parent directory is writable (for atomic rename)
        if let Some(parent) = current_exe.parent() {
            let parent_meta = fs::metadata(parent)?;
            let mode = parent_meta.mode();
            // Check write permission for owner
            if mode & 0o200 == 0 {
                // No write permission, clean up staging and skip
                let _ = fs::remove_dir_all(&staging);
                return Ok(false);
            }
        }
    }

    // Read version info
    let new_version = fs::read_to_string(&version_file)
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let current_version = env!("CARGO_PKG_VERSION");

    // Verify the staged binary exists and is executable
    let pending_meta = fs::metadata(&pending_binary)?;
    if pending_meta.len() == 0 {
        // Empty file, clean up and skip
        let _ = fs::remove_dir_all(&staging);
        return Ok(false);
    }

    // Atomic replace: rename pending binary over current
    // On Unix, this is atomic if same filesystem
    if let Err(e) = fs::rename(&pending_binary, &current_exe) {
        // Rename failed (possibly different filesystem), try copy + delete
        if let Err(_) = fs::copy(&pending_binary, &current_exe) {
            // Copy also failed, clean up and continue with old binary
            let _ = fs::remove_dir_all(&staging);
            return Err(e.into());
        }
    }

    // Set executable permission (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&current_exe, perms)?;
    }

    // Cleanup staging directory
    let _ = fs::remove_dir_all(&staging);

    // Notify user
    eprintln!(
        "{} Updated proj {} {} {}",
        "✓".green(),
        current_version.dimmed(),
        "→".dimmed(),
        new_version.green()
    );

    // Re-execute with same arguments
    let args: Vec<String> = std::env::args().collect();
    let status = Command::new(&current_exe)
        .args(&args[1..])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to re-execute after update: {}", e))?;

    std::process::exit(status.code().unwrap_or(0));
}
