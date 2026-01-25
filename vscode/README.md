# proj VS Code Extension

A VS Code extension that integrates **proj** project tracking with GitHub Copilot Chat. Track decisions, tasks, blockers, and session context directly from your editor.

## Table of Contents

- [What Is This Extension?](#what-is-this-extension)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start Guide](#quick-start-guide)
- [Features In Detail](#features-in-detail)
  - [Session Notification](#1-session-notification)
  - [Status Bar Menu](#2-status-bar-menu)
  - [Chat Participant (@proj)](#3-chat-participant-proj)
  - [Automatic Logging with Language Model Tools](#4-automatic-logging-with-language-model-tools)
  - [Ending Sessions](#5-ending-sessions)
- [Understanding Permissions](#understanding-permissions)
- [CLI vs VS Code: What's the Difference?](#cli-vs-vs-code-whats-the-difference)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [How It Works (Technical Details)](#how-it-works-technical-details)
- [Development](#development)
- [Version History](#version-history)

---

## What Is This Extension?

When you work on a coding project, you make lots of decisions, create tasks, and encounter blockers. This extension helps you **track all of that automatically** so you (or an AI assistant) can remember what happened later.

**What it does:**

1. **Shows you where you left off** - When you open VS Code, you see a notification with your session status
2. **Lets you ask questions** - Type `@proj` in Copilot Chat to ask about your project
3. **Logs things automatically** - When you tell Copilot "let's use SQLite for the database", it can automatically log that decision
4. **Provides quick access** - Click the status bar for a menu with common actions like ending your session

**Why use it?**

- Never forget why you made a decision
- Pick up exactly where you left off, even weeks later
- AI assistants can read your project history and give better help

---

## Requirements

Before you can use this extension, you need three things:

### 1. Install the proj CLI

The proj command-line tool stores all your project data. Choose one of these installation methods:

#### Option A: Cargo / crates.io (All Platforms)

If you have Rust installed:

```bash
cargo install aiproject
```

This works on macOS, Linux, and Windows.

#### Option B: Homebrew (macOS/Linux)

```bash
brew tap victorysightsound/tap
brew install aiproject
```

#### Option C: Download from GitHub Releases

1. Go to https://github.com/victorysightsound/aiproject/releases/latest
2. Download the file for your system:
   - **macOS Apple Silicon (M1/M2/M3):** `proj-aarch64-apple-darwin.tar.gz`
   - **macOS Intel:** `proj-x86_64-apple-darwin.tar.gz`
   - **Linux x64:** `proj-x86_64-unknown-linux-gnu.tar.gz`
   - **Linux ARM:** `proj-aarch64-unknown-linux-gnu.tar.gz`
   - **Windows:** `proj-x86_64-pc-windows-msvc.zip`
3. Extract and move to your PATH:

```bash
# macOS/Linux example:
tar -xzf proj-*.tar.gz
sudo mv proj /usr/local/bin/
```

#### Option D: Build from Source

Requires Rust 1.70+:

```bash
git clone https://github.com/victorysightsound/aiproject.git
cd aiproject
cargo build --release
sudo cp target/release/proj /usr/local/bin/
```

#### Verify Installation

```bash
proj --version
```

You should see a version number like `1.6.1`.

### 2. Initialize proj in your project

Navigate to your project folder and run:

```bash
cd /path/to/your/project
proj init
```

This creates a `.tracking` folder with a SQLite database that stores all your project data.

### 3. Install GitHub Copilot

The chat features require GitHub Copilot. Install it from the VS Code marketplace:

1. Open VS Code
2. Go to Extensions (Cmd+Shift+X on Mac, Ctrl+Shift+X on Windows)
3. Search for "GitHub Copilot"
4. Click Install
5. Sign in with your GitHub account

---

## Installation

### Option A: From VS Code Marketplace (Easiest)

1. Open VS Code
2. Go to Extensions (Cmd+Shift+X on Mac, Ctrl+Shift+X on Windows)
3. Search for "proj - AI Project Tracker"
4. Click Install
5. Restart VS Code

### Option B: From VSIX File

If you have a `.vsix` file:

1. Open VS Code
2. Go to Extensions
3. Click the `...` menu at the top right of the Extensions panel
4. Select "Install from VSIX..."
5. Choose the `.vsix` file
6. Restart VS Code

### Option C: Build from Source

If you want to build it yourself:

```bash
cd vscode
npm install
npm run compile
npm run package
code --install-extension proj-1.6.1.vsix
```

---

## Quick Start Guide

Here's how to start using the extension in 5 minutes:

### Step 1: Open a tracked project

Open a folder in VS Code that has proj tracking (you ran `proj init` in it). You'll know it's tracked if it has a `.tracking` folder.

### Step 2: See the notification

After about 1.5 seconds, you'll see a notification in the bottom-right corner:

```
proj: MyProject | Session #5 | 2 tasks
Last: Fixed authentication bug and added tests
[View Full Status]  [End Session]  [OK]
```

Click **OK** to dismiss it, or click one of the other buttons for more options.

### Step 3: Try the status bar

Look at the bottom of VS Code. You'll see something like:

```
proj (#5, 2 tasks)
```

**Click it** to open a menu with options:
- View Status
- View Tasks
- End Session...
- Refresh

### Step 4: Try Copilot Chat

Open the Copilot Chat panel (click the Copilot icon in the sidebar or press Cmd+Shift+I).

Type: `@proj /status`

You'll see your project status with tasks, blockers, and recent decisions.

### Step 5: Let Copilot log automatically

In Copilot Chat (without @proj), just have a normal conversation:

> You: "I think we should use PostgreSQL instead of MySQL for the database"

Copilot may ask: "Would you like me to log this decision?"

Click **Allow** and it will be saved to your project history.

---

## Features In Detail

### 1. Session Notification

**What it is:** A popup that appears when you open VS Code in a tracked project.

**When it appears:** About 1.5 seconds after VS Code opens (to make sure you see it).

**What it shows:**
- Project name
- Current session number
- Number of active tasks
- Number of blockers (if any)
- Summary of your last session

**The buttons:**

| Button | What it does |
|--------|--------------|
| View Full Status | Opens the Output panel with detailed project status |
| End Session | Opens options to end your session with a summary |
| OK | Dismisses the notification |

**Important:** The notification stays on screen until you click a button. It won't disappear on its own.

---

### 2. Status Bar Menu

**What it is:** A clickable item in the status bar at the bottom of VS Code.

**What it shows:** `proj (#5, 2 tasks)` - your session number and task count.

**How to use it:** Click it to open a quick menu.

**Menu options:**

| Option | What it does |
|--------|--------------|
| View Status | Shows full project status in the Output panel |
| View Tasks | Shows all your active tasks |
| End Session... | Opens options to end your session |
| Refresh | Updates the status bar with current data |

**Why it's useful:** One click to end your session - no typing required!

---

### 3. Chat Participant (@proj)

**What it is:** A way to talk to your project tracking directly in Copilot Chat.

**How to use it:** Start your message with `@proj` in the Copilot Chat panel.

#### Slash Commands

These are shortcuts for common actions:

| Command | What it does | Example |
|---------|--------------|---------|
| `@proj /status` | Shows current project status | Just type it |
| `@proj /tasks` | Lists all your tasks | Just type it |
| `@proj /decisions` | Shows recent decisions | Just type it |
| `@proj /log` | Shows how to log things | Just type it |
| `@proj /end` | Ends session with a summary | `@proj /end Fixed the login bug` |

#### Natural Language

You can also just ask questions naturally:

- `@proj where did I leave off?` - Shows your status
- `@proj what are my tasks?` - Lists your tasks
- `@proj what did we decide about the database?` - Searches your decisions
- `@proj am I blocked on anything?` - Shows your blockers

#### Logging Things Manually

Use `/log` to record things:

```
@proj /log decision "database" "Using PostgreSQL" "Better performance for our needs"
```

```
@proj /log blocker "Waiting for API credentials from the client"
```

```
@proj /log note "constraint" "API Rate Limit" "Maximum 100 requests per minute"
```

---

### 4. Automatic Logging with Language Model Tools

**What it is:** The extension gives Copilot special "tools" it can use to log things automatically during your conversation.

**How it works:**

1. You're chatting with Copilot (not using @proj, just normal chat)
2. You say something like "Let's use Redis for caching"
3. Copilot recognizes this is a decision
4. Copilot asks: "Would you like me to log this decision?"
5. You click Allow
6. The decision is saved to your project history

**The tools available to Copilot:**

| Tool | When Copilot uses it |
|------|----------------------|
| `proj_get_status` | When you ask about your project |
| `proj_log_decision` | When you make a technical choice |
| `proj_add_task` | When you mention something to do later |
| `proj_log_blocker` | When you say you're stuck on something |
| `proj_log_note` | When you share important context |
| `proj_update_task` | When you complete or start a task |
| `proj_end_session` | When you say you're done working |
| `proj_search_context` | When you ask about past decisions |
| `proj_get_session_activity` | When generating session summaries |

**Examples of automatic logging:**

> You: "I decided to use TypeScript instead of JavaScript for better type safety"
>
> Copilot: *asks to log decision* → Logs: "language" - "Using TypeScript instead of JavaScript" - "Better type safety"

> You: "I need to add error handling to the API later"
>
> Copilot: *asks to add task* → Adds task: "Add error handling to the API"

> You: "I'm stuck because the staging server is down"
>
> Copilot: *asks to log blocker* → Logs blocker: "Staging server is down"

---

### 5. Ending Sessions

When you're done working, you should end your session with a summary. This helps you remember what you did when you come back later.

**Three ways to end a session:**

#### Option A: Status Bar Menu (Easiest)

1. Click the status bar item (`proj (#5)`)
2. Select "End Session..."
3. Choose:
   - **Enter summary manually** - Type your own summary
   - **Auto-generate summary** - Let Copilot write it for you

#### Option B: Notification Button

When you first open VS Code, click the "End Session" button on the notification.

#### Option C: Chat Command

Type in Copilot Chat:
```
@proj /end Implemented user authentication and fixed the login bug
```

#### Auto-Generated Summaries

When you choose "Auto-generate summary":

1. Copilot opens in a new chat
2. Copilot looks at what you did (decisions, tasks, git changes)
3. Copilot writes a summary like: "Modified VS Code extension files and added tools.ts"
4. Copilot ends the session with that summary

This saves you from having to remember and type everything yourself!

---

## Understanding Permissions

When Copilot wants to use a proj tool, VS Code asks for your permission. This is a security feature.

**The permission dialog looks like:**

```
Copilot wants to use: proj_log_decision
[Allow] [Allow for this Session] [Always Allow] [Deny]
```

**What each option means:**

| Option | What it does |
|--------|--------------|
| Allow | Allows this one time only. Asks again next time. |
| Allow for this Session | Allows for the rest of this VS Code session. Won't ask again until you restart VS Code. |
| Always Allow | Permanently allows. Never asks again for this tool. |
| Deny | Blocks this one time. |

**Recommended approach:**

For the smoothest experience, choose **"Allow for this Session"** the first time each tool is used. Then Copilot will log things automatically without asking every time.

**Tools you'll want to allow:**
- `proj_log_decision` - For logging decisions
- `proj_add_task` - For adding tasks
- `proj_log_blocker` - For logging blockers
- `proj_log_note` - For logging notes
- `proj_get_status` - For checking status
- `proj_end_session` - For ending sessions
- `proj_get_session_activity` - For auto-generating summaries

---

## CLI vs VS Code: What's the Difference?

You might wonder: should I use proj in the terminal or in VS Code?

### Terminal (CLI) with Claude Code

| Aspect | How it works |
|--------|--------------|
| Session start | Automatic when you run `proj status` |
| Logging | Automatic, no prompts or confirmations |
| Speed | Fastest - no permission dialogs |
| When to use | When you want everything logged automatically |

### VS Code with Copilot

| Aspect | How it works |
|--------|--------------|
| Session start | Automatic when you open the workspace |
| Logging | Asks permission before each action (unless you allowed for session) |
| Speed | Slightly slower due to confirmations |
| When to use | When working in VS Code and want visual feedback |

**Key differences:**

1. **Permissions**: VS Code asks before logging (security feature). CLI doesn't.
2. **Consistency**: CLI always logs when it detects something. Copilot's judgment varies.
3. **Visual feedback**: VS Code shows notifications and buttons. CLI is text-only.

**They work together!** Both use the same database (`.tracking/tracking.db`), so you can switch between them freely. What you log in VS Code appears in CLI and vice versa.

---

## Configuration

Open VS Code Settings (Cmd+, on Mac, Ctrl+, on Windows) and search for "proj".

| Setting | Default | What it does |
|---------|---------|--------------|
| `proj.cliPath` | `/usr/local/bin/proj` | Where the proj command is installed |
| `proj.showStatusBar` | `true` | Show/hide the status bar item |

### Finding your proj path

If proj is installed somewhere else, find it:

```bash
which proj
```

Then update `proj.cliPath` in settings to match.

---

## Troubleshooting

### Extension not showing up

**Problem:** No status bar item, no notifications.

**Solutions:**
1. Make sure the workspace has `.tracking/tracking.db` (run `proj init` first)
2. Restart VS Code completely (Cmd+Q, then reopen)
3. Check if the extension is installed: Extensions → search "proj"

### "proj CLI not found" error

**Problem:** Extension can't find the proj command.

**Solutions:**
1. Make sure proj is installed: `which proj`
2. Update `proj.cliPath` in settings to the full path (e.g., `/usr/local/bin/proj`)

### Commands hang or timeout

**Problem:** Running commands takes forever or fails.

**Solution:** The extension needs the full path to proj. Update `proj.cliPath` in settings:
1. Run `which proj` in terminal
2. Copy the path (e.g., `/usr/local/bin/proj`)
3. Open Settings → search "proj.cliPath" → paste the path

### Notification doesn't appear

**Problem:** No popup when opening VS Code.

**Solutions:**
1. Wait 1.5 seconds - it delays intentionally
2. Check if you're in a tracked project (has `.tracking` folder)
3. Look in the bottom-right corner of VS Code
4. Try restarting VS Code

### Copilot not using proj tools

**Problem:** Copilot doesn't offer to log things.

**Solutions:**
1. Make sure GitHub Copilot is installed and you're signed in
2. The tools only work in Copilot Chat (not inline code completions)
3. Try being explicit: "Please log this decision using the proj tools"
4. Make sure you're in a workspace with proj tracking

### Status bar not updating

**Problem:** Status bar shows old information.

**Solutions:**
1. Click the status bar → select "Refresh"
2. Or run command: `proj: Refresh`

---

## How It Works (Technical Details)

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        VS Code                               │
│  ┌──────────────────┐  ┌──────────────────────────────────┐ │
│  │   Status Bar     │  │     GitHub Copilot Chat          │ │
│  │  (click for menu)│  │  ┌───────────────────────────┐   │ │
│  └────────┬─────────┘  │  │  @proj Chat Participant   │   │ │
│           │            │  │  - /status, /tasks, etc.  │   │ │
│           │            │  └───────────────────────────┘   │ │
│           │            │  ┌───────────────────────────┐   │ │
│           │            │  │  Language Model Tools     │   │ │
│           │            │  │  (automatic logging)      │   │ │
│           │            │  └───────────────────────────┘   │ │
│           │            └──────────────────────────────────┘ │
│           │                          │                       │
│           └──────────────┬───────────┘                       │
│                          │                                   │
│                          ▼                                   │
│               ┌─────────────────────┐                        │
│               │      CLI Module     │                        │
│               │  (runs proj commands│                        │
│               └──────────┬──────────┘                        │
└──────────────────────────┼───────────────────────────────────┘
                           │
                           ▼
                ┌─────────────────────┐
                │     proj CLI        │
                │  /usr/local/bin/proj│
                └──────────┬──────────┘
                           │
                           ▼
                ┌─────────────────────┐
                │  .tracking/         │
                │  tracking.db        │
                │  (SQLite database)  │
                └─────────────────────┘
```

### File Structure

```
vscode/
├── package.json        # Extension manifest and tool declarations
├── tsconfig.json       # TypeScript configuration
├── README.md           # This documentation
├── LICENSE             # MIT License
├── src/
│   ├── extension.ts    # Main entry point, commands, notifications
│   ├── cli.ts          # Runs proj CLI commands
│   ├── participant.ts  # @proj chat participant
│   ├── statusBar.ts    # Status bar item and menu
│   └── tools.ts        # Language Model Tools for Copilot
└── out/                # Compiled JavaScript (generated)
```

### How the pieces connect

1. **extension.ts** - Runs when VS Code opens a tracked workspace. Shows notification, registers commands.

2. **statusBar.ts** - Creates the status bar item. Updates every 60 seconds. Opens menu when clicked.

3. **participant.ts** - Handles `@proj` messages in Copilot Chat. Routes to appropriate commands.

4. **tools.ts** - Defines tools that Copilot can call automatically during conversation.

5. **cli.ts** - Wrapper that executes the proj command-line tool and returns results.

---

## Development

### Building from Source

```bash
cd vscode
npm install
npm run compile
```

### Watching for Changes

```bash
npm run watch
```

Press F5 in VS Code to launch Extension Development Host for testing.

### Packaging

```bash
npm run package
```

Creates a `.vsix` file you can install or distribute.

### Installing after changes

```bash
npm run package
code --install-extension proj-1.6.1.vsix --force
```

Then restart VS Code.

---

## Version History

### 1.6.1

- Improved session summary guidance: encourage 1-3 substantive sentences vs generic summaries
- Updated CLI help text and documentation

### 1.5.3

- Added crates.io installation option (`cargo install aiproject`)
- Reordered installation methods (cargo first as cross-platform option)

### 1.5.2

- Updated CLI installation documentation with multiple methods (Homebrew, GitHub releases, build from source)

### 1.5.1

- Added custom extension icon (clipboard with progress indicators)

### 1.5.0

- Added Language Model Tools for automatic logging (9 tools)
- Added session notification on workspace open with action buttons
- Notification shows last session summary and stays until dismissed
- Copilot can automatically log decisions, tasks, and blockers during conversation
- Added "End Session" button to status output in Copilot Chat
- Added auto-generate summary option for ending sessions
- Copilot can generate session summaries based on actual activity
- Status bar now opens a quick menu when clicked
- One-click access to End Session from status bar menu

### 1.4.0

- Initial release
- Chat participant with slash commands
- Status bar integration
- Natural language queries

---

## License

MIT License - see LICENSE file.

## Support

- **Issues:** https://github.com/victorysightsound/aiproject/issues
- **Documentation:** https://github.com/victorysightsound/aiproject
