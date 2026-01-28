// Shell integration commands - install/uninstall shell hooks for automatic tracking

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

/// Shell hook marker - used to detect if hook is already installed
const HOOK_MARKER_START: &str = "# >>> proj shell integration >>>";
const HOOK_MARKER_END: &str = "# <<< proj shell integration <<<";

/// Zsh hook code - uses chpwd which fires on directory change
const ZSH_HOOK: &str = r#"# >>> proj shell integration >>>
# Automatically runs proj enter when cd'ing into a project with tracking
_proj_auto_enter() {
    if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
        proj enter
    fi
}
# Add to chpwd hooks if not already present
if [[ -z "${chpwd_functions[(r)_proj_auto_enter]}" ]]; then
    chpwd_functions+=(_proj_auto_enter)
fi
# <<< proj shell integration <<<"#;

/// Bash hook code - uses PROMPT_COMMAND with directory tracking
const BASH_HOOK: &str = r#"# >>> proj shell integration >>>
# Automatically runs proj enter when cd'ing into a project with tracking
_proj_last_dir=""
_proj_auto_enter() {
    if [[ "$PWD" != "$_proj_last_dir" ]]; then
        _proj_last_dir="$PWD"
        if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
            proj enter
        fi
    fi
}
# Add to PROMPT_COMMAND if not already present
if [[ "$PROMPT_COMMAND" != *"_proj_auto_enter"* ]]; then
    PROMPT_COMMAND="_proj_auto_enter${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
# <<< proj shell integration <<<"#;

pub fn install(force: bool) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    // Check what shells are available
    let zsh_exists = zshrc.exists();
    let bash_exists = bashrc.exists();

    if !zsh_exists && !bash_exists {
        println!(
            "{} No shell config files found (.zshrc or .bashrc)",
            "⚠".yellow()
        );
        println!("Please create ~/.zshrc or ~/.bashrc and run this command again.");
        return Ok(());
    }

    // Check if already installed
    let zsh_installed = zsh_exists && is_hook_installed(&zshrc)?;
    let bash_installed = bash_exists && is_hook_installed(&bashrc)?;

    if zsh_installed || bash_installed {
        println!("{} Shell integration already installed:", "✓".green());
        if zsh_installed {
            println!("  • ~/.zshrc");
        }
        if bash_installed {
            println!("  • ~/.bashrc");
        }
        if !force {
            println!();
            println!("To reinstall, first run: proj shell uninstall");
        }
        return Ok(());
    }

    if !force {
        println!("{}", "Shell Integration Setup".bold());
        println!();

        // Show what we're about to do
        println!("This will add a hook to your shell configuration that:");
        println!("  • Detects when you cd into a directory with .tracking/");
        println!("  • Automatically runs 'proj enter' to start/continue session");
        println!("  • Shows project context only when starting a new session");
        println!();
    }

    let mut installed_shells = Vec::new();

    // Install for zsh
    if zsh_exists && !zsh_installed {
        let should_install = force
            || Confirm::new()
                .with_prompt("Install for zsh (~/.zshrc)?")
                .default(true)
                .interact()?;

        if should_install {
            install_hook(&zshrc, ZSH_HOOK)?;
            installed_shells.push("zsh");
            println!("{} Installed in ~/.zshrc", "✓".green());
        }
    }

    // Install for bash
    if bash_exists && !bash_installed {
        let should_install = force
            || Confirm::new()
                .with_prompt("Install for bash (~/.bashrc)?")
                .default(true)
                .interact()?;

        if should_install {
            install_hook(&bashrc, BASH_HOOK)?;
            installed_shells.push("bash");
            println!("{} Installed in ~/.bashrc", "✓".green());
        }
    }

    if installed_shells.is_empty() {
        println!("No changes made.");
        return Ok(());
    }

    println!();
    println!("{}", "Setup complete!".bold().green());
    println!();
    println!("To activate, either:");
    println!("  • Open a new terminal window, or");
    println!("  • Run: source ~/.{}rc", installed_shells[0]);
    println!();
    println!("Then cd into any project with proj tracking - session will start automatically.");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    println!("{}", "Removing Shell Integration".bold());
    println!();

    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    let mut removed = false;

    // Remove from zsh
    if zshrc.exists() && is_hook_installed(&zshrc)? {
        remove_hook(&zshrc)?;
        println!("{} Removed from ~/.zshrc", "✓".green());
        removed = true;
    }

    // Remove from bash
    if bashrc.exists() && is_hook_installed(&bashrc)? {
        remove_hook(&bashrc)?;
        println!("{} Removed from ~/.bashrc", "✓".green());
        removed = true;
    }

    if removed {
        println!();
        println!("Shell integration removed. Open a new terminal for changes to take effect.");
    } else {
        println!("Shell integration was not installed.");
    }

    Ok(())
}

pub fn status() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    println!("{}", "Shell Integration Status".bold());
    println!();

    let zsh_installed = zshrc.exists() && is_hook_installed(&zshrc)?;
    let bash_installed = bashrc.exists() && is_hook_installed(&bashrc)?;

    if zsh_installed || bash_installed {
        println!("{} Shell integration is installed:", "✓".green());
        if zsh_installed {
            println!("  • ~/.zshrc");
        }
        if bash_installed {
            println!("  • ~/.bashrc");
        }
    } else {
        println!("{} Shell integration is not installed.", "○".white());
        println!();
        println!("Run 'proj shell install' to enable automatic session tracking.");
    }

    Ok(())
}

/// Check if hook is already installed in a file
fn is_hook_installed(path: &PathBuf) -> Result<bool> {
    let content = fs::read_to_string(path).unwrap_or_default();
    Ok(content.contains(HOOK_MARKER_START))
}

/// Install hook at the end of a shell config file
fn install_hook(path: &PathBuf, hook: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_default();

    // Add a newline if the file doesn't end with one
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(hook);
    content.push('\n');

    fs::write(path, content).with_context(|| format!("Failed to write to {:?}", path))?;
    Ok(())
}

/// Remove hook from a shell config file
fn remove_hook(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)?;

    // Find and remove the hook block
    let mut new_content = String::new();
    let mut in_hook_block = false;

    for line in content.lines() {
        if line.contains(HOOK_MARKER_START) {
            in_hook_block = true;
            continue;
        }
        if line.contains(HOOK_MARKER_END) {
            in_hook_block = false;
            continue;
        }
        if !in_hook_block {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Clean up any double blank lines
    while new_content.contains("\n\n\n") {
        new_content = new_content.replace("\n\n\n", "\n\n");
    }

    fs::write(path, new_content).with_context(|| format!("Failed to write to {:?}", path))?;
    Ok(())
}

/// Check if shell integration is installed (for use by other commands)
pub fn is_installed() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    is_hook_installed(&zshrc).unwrap_or(false) || is_hook_installed(&bashrc).unwrap_or(false)
}
