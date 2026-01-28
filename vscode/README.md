# proj VS Code Extension

A VS Code extension that integrates **proj** project tracking with GitHub Copilot Chat. Track decisions, tasks, blockers, and session context directly from your editor.

## Table of Contents

- [What Is This Extension?](#what-is-this-extension)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start Guide](#quick-start-guide)
- [Using @proj in Copilot Chat](#using-proj-in-copilot-chat)
  - [Three Ways to Use proj](#three-ways-to-use-proj-in-copilot)
  - [Understanding Copilot Modes](#understanding-copilot-modes)
  - [Recommended Workflow](#recommended-workflow)
  - [Quick Reference](#quick-reference)
  - [When Things Don't Work](#when-things-dont-work)
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

> **What to Expect:** The `@proj` commands (like `@proj /status` or `@proj /log`) work reliably in any Copilot mode. Automatic logging (where Copilot detects and logs decisions on its own) depends on which Copilot mode you're in and which language model is active -- it may or may not happen. When in doubt, use `@proj` commands directly. See [Using @proj in Copilot Chat](#using-proj-in-copilot-chat) for the full picture.

---

## Requirements

Before you can use this extension, you need three things:

### 1. Install the proj CLI

The proj command-line tool stores all your project data. Choose one of these installation methods:

#### Option A: npm (Easiest)

If you have Node.js installed:

```bash
npm install -g create-aiproj
```

This downloads a pre-built binary for your platform. No Rust required. Works on macOS, Linux, and Windows.

#### Option B: Cargo / crates.io (All Platforms)

If you have Rust installed:

```bash
cargo install aiproject
```

Compiles from source. Works on macOS, Linux, and Windows.

#### Option C: Homebrew (macOS/Linux)

```bash
brew tap victorysightsound/tap
brew install aiproject
```

#### Option D: Download from GitHub Releases

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

#### Option E: Build from Source

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

You should see a version number like `1.7.28`.

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

**Direct link:** [proj - AI Project Tracker on Marketplace](https://marketplace.visualstudio.com/items?itemName=victorysightsound.proj)

**Or install from VS Code:**
1. Open VS Code
2. Go to Extensions (Cmd+Shift+X on Mac, Ctrl+Shift+X on Windows)
3. Search for "proj - AI Project Tracker" or `victorysightsound.proj`
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
code --install-extension proj-1.7.28.vsix
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

### Step 5: Try automatic logging (bonus)

In Copilot Chat **Ask mode** (without @proj), try a normal conversation:

> You: "I think we should use PostgreSQL instead of MySQL for the database"

Copilot **may** ask: "Would you like me to log this decision?" If it does, click **Allow** and it will be saved to your project history.

**Note:** Automatic logging only works in Ask mode and depends on the language model. It won't always happen. If it doesn't, just use `@proj` directly -- that always works. See [Using @proj in Copilot Chat](#using-proj-in-copilot-chat) for the full guide.

---

## Using @proj in Copilot Chat

`@proj` is the most reliable way to interact with your project tracking from Copilot Chat. When you type `@proj`, you're talking directly to the proj chat participant -- it handles your request itself, so it works regardless of which Copilot mode you're in or which language model is active.

This section covers everything you need to know about using proj in Copilot Chat, organized from most reliable to least reliable.

### Three Ways to Use proj in Copilot

#### 1. @proj Slash Commands (Always Works)

Slash commands are direct shortcuts. Type `@proj` followed by the command. These always produce results, in any Copilot mode.

| Command | What it does | Example |
|---------|--------------|---------|
| `/status` | Shows project status, session info, tasks, blockers | `@proj /status` |
| `/tasks` | Lists all your tasks with status and priority | `@proj /tasks` |
| `/decisions` | Shows recent decisions you've logged | `@proj /decisions` |
| `/log` | Logs a decision, task, blocker, or note | `@proj /log decision "database" "Using SQLite" "Simpler for our needs"` |
| `/end` | Ends session with your summary | `@proj /end Fixed the login bug and added tests` |
| `/end-auto` | Ends session with an AI-generated summary | `@proj /end-auto` |

**Examples:**

```
@proj /status
```
Shows your current project state -- session number, active tasks, blockers, and last session summary.

```
@proj /log decision "auth" "Using JWT tokens" "Stateless, works with our API gateway"
```
Logs a decision with topic, what was decided, and why.

```
@proj /log blocker "Waiting for API credentials from the client"
```
Logs a blocker.

```
@proj /end Refactored the authentication module and wrote unit tests
```
Ends your session with a specific summary.

#### 2. @proj Natural Language (Always Works)

You can also type `@proj` followed by a question or plain statement. The participant figures out what you mean and either routes to the right command, searches your project context, or auto-detects decisions/tasks/blockers.

**Asking questions:**

- `@proj where did I leave off?` -- Shows your status and last session summary
- `@proj what are my tasks?` -- Lists active tasks
- `@proj what did we decide about the database?` -- Searches your decision history
- `@proj am I blocked on anything?` -- Shows current blockers

**Auto-detection (logging by conversation):**

When your message contains a decision, task, or blocker, the participant detects and logs it automatically:

> You: `@proj Let's use TypeScript instead of JavaScript for better type safety`
>
> @proj: Logged decision: typescript -- Using TypeScript instead of JavaScript

> You: `@proj I need to add error handling to the API later`
>
> @proj: Added task: Add error handling to the API

> You: `@proj I'm stuck because the staging server is down`
>
> @proj: Logged blocker: Staging server is down

This works in any Copilot mode because the @proj participant handles it directly.

#### 3. Automatic Language Model Tools (Sometimes Works)

Without the `@proj` prefix, Copilot itself may recognize decisions, tasks, or blockers and call proj tools on your behalf. This is the least reliable method:

- **Only works in Ask mode** -- Agent mode does not support extension-provided Language Model Tools
- **Depends on the model** -- Some models are better at recognizing when to use tools
- **Requires permission clicks** -- VS Code shows a permission dialog each time (unless you chose "Always Allow")
- **May not happen at all** -- Copilot may simply answer your question without calling any tools

**When it works, it looks like this:**

1. You say something like "Let's use Redis for caching" (without @proj)
2. Copilot recognizes this as a decision
3. VS Code asks: "Copilot wants to use: proj_log_decision" with Allow/Deny buttons
4. You click Allow
5. The decision is saved

**Bottom line:** Think of automatic logging as a nice bonus when it happens, not something to rely on. Always use `@proj` when you want to make sure something gets logged.

### Understanding Copilot Modes

Copilot Chat has different modes that affect how proj works:

| Mode | @proj Commands | @proj Natural Language | Automatic Language Model Tools | Notes |
|------|:-:|:-:|:-:|-------|
| **Ask mode** | Yes | Yes | Yes | Best mode for proj. All features work. |
| **Agent mode** (Copilot Edits) | Yes | Yes | No | @proj still works, but Copilot won't call proj tools on its own. Agent mode may try to edit files instead. |
| **Inline completions** | N/A | N/A | N/A | Code completions only -- no chat interaction. |

**How to check your mode:** Look at the top of the Copilot Chat panel. You'll see a dropdown or toggle that says "Ask" or "Agent" (or similar). For the best proj experience, use **Ask mode**.

### Recommended Workflow

Here's the practical approach that works every time:

1. **Start your session:** Open VS Code in a tracked project. The notification appears with your status. Click OK or View Full Status.

2. **Check where you left off:** Type `@proj /status` in Copilot Chat.

3. **Work normally.** Write code, debug, design -- whatever you need to do.

4. **Log as you go.** When you make a decision, identify a task, or hit a blocker, use `@proj`:
   - `@proj /log decision "routing" "Using file-based routing" "Matches Next.js conventions"`
   - `@proj I need to refactor the auth middleware later`
   - `@proj /log blocker "CI pipeline is broken, can't deploy"`

5. **If Copilot logs automatically, great.** You may see permission dialogs pop up during regular conversation (Ask mode). Allow them -- it's a bonus. But don't count on it.

6. **End your session:** Click the status bar and select "End Session", or type `@proj /end-auto` to let AI summarize what you did.

### Quick Reference

| I want to... | Do this |
|---------------|---------|
| See my project status | `@proj /status` |
| List my tasks | `@proj /tasks` |
| See recent decisions | `@proj /decisions` |
| Log a decision | `@proj /log decision "topic" "what" "why"` |
| Add a task | `@proj I need to do X later` |
| Log a blocker | `@proj /log blocker "description"` |
| Add a note | `@proj /log note "category" "title" "content"` |
| Search project history | `@proj what did we decide about X?` |
| End session (manual summary) | `@proj /end Did X, fixed Y, updated Z` |
| End session (auto summary) | `@proj /end-auto` |
| End session (status bar) | Click `proj (#5)` in status bar, select "End Session..." |

### When Things Don't Work

**"I typed @proj but nothing happened"**
- Make sure the extension is installed (check Extensions panel)
- Make sure you're in a workspace with a `.tracking` folder
- Restart VS Code if needed

**"Copilot isn't logging things automatically"**
- This is normal. Automatic logging only works in Ask mode and depends on the model.
- Switch to Ask mode if you're in Agent mode
- Use `@proj` directly -- it's more reliable

**"I see permission dialogs every time"**
- Choose "Allow for this Session" or "Always Allow" to stop repeated prompts
- See [Understanding Permissions](#understanding-permissions) for details

**"@proj gives an error about the CLI"**
- Run `which proj` in your terminal to find the path
- Update `proj.cliPath` in VS Code settings to match
- See [Troubleshooting](#troubleshooting) for more

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
| View Full Status | Opens `@proj /status` in Copilot Chat |
| End Session | Opens `@proj /end-auto` in Copilot Chat |
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
| View Status | Opens `@proj /status` in Copilot Chat |
| View Tasks | Opens `@proj /tasks` in Copilot Chat |
| End Session... | Opens `@proj /end-auto` in Copilot Chat |
| Refresh | Updates the status bar with current data |

**Why it's useful:** One click opens Copilot Chat with the right command - no typing required!

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
| `@proj /end-auto` | Ends session with AI-generated summary | Just type it |

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

The extension provides two ways to automatically log decisions, tasks, and blockers:

#### Method A: Copilot Ask Mode (Language Model Tools)

The extension registers tools that Copilot can call during **Ask mode** conversations. When Copilot recognizes a decision, task, or blocker, it asks for permission before logging.

1. Open Copilot Chat in **Ask mode** (not Agent mode)
2. Say something like "Let's use Redis for caching"
3. Copilot recognizes this is a decision
4. Copilot asks: "Would you like me to log this decision?"
5. You click Allow
6. The decision is saved to your project history

**Important:** This only works in Copilot **Ask mode**. Agent mode (which can edit files) does not support extension-provided Language Model Tools.

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

#### Method B: @proj Auto-Detection

When chatting with `@proj` (any mode), the participant analyzes your messages using AI and automatically detects and logs decisions, tasks, and blockers. No permission dialog needed.

> You: `@proj Let's use TypeScript instead of JavaScript for better type safety`
>
> @proj: `> Logged decision: typescript — Using TypeScript instead of JavaScript`

> You: `@proj I need to add error handling to the API later`
>
> @proj: `> Added task: Add error handling to the API`

> You: `@proj I'm stuck because the staging server is down`
>
> @proj: `> Logged blocker: Staging server is down`

This works in any Copilot mode because it runs through the @proj participant, not through Copilot's general tool system.

---

### 5. Ending Sessions

When you're done working, you should end your session with a summary. This helps you remember what you did when you come back later.

**Four ways to end a session:**

#### Option A: Status Bar Menu (Easiest)

1. Click the status bar item (`proj (#5)`)
2. Select "End Session..."
3. Copilot Chat opens with `@proj /end-auto` to generate a summary automatically

#### Option B: Notification Button

When you first open VS Code, click the "End Session" button on the notification. This opens Copilot Chat with `@proj /end-auto`.

#### Option C: Chat Command (Manual Summary)

Type in Copilot Chat:
```
@proj /end Implemented user authentication and fixed the login bug
```

#### Option D: Chat Command (Auto Summary)

Type in Copilot Chat:
```
@proj /end-auto
```

This uses AI to review your session activity (decisions, tasks, git changes) and generate a summary automatically.

#### How Auto-Generated Summaries Work

When you use `/end-auto` (from the menu, notification, or chat):

1. The extension reviews all session activity (decisions, tasks, blockers, git commits)
2. An AI model generates a concise summary of what was accomplished
3. The session ends with that summary saved for future reference

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

**Problem:** Copilot doesn't offer to log decisions, tasks, or blockers automatically.

**Understanding Copilot modes:**
- **Ask mode**: Language Model Tools work here. Copilot can call `proj_log_decision`, `proj_add_task`, etc. and will ask for your permission.
- **Agent mode** (Copilot Edits): Does NOT support extension-provided Language Model Tools. Agent mode can edit files but cannot call proj tools.

**Solutions:**
1. Make sure you're using **Ask mode** in Copilot Chat (not Agent mode)
2. Make sure GitHub Copilot is installed and you're signed in
3. The tools only work in Copilot Chat (not inline code completions)
4. Try being explicit: "Please log this decision using the proj tools"
5. Make sure you're in a workspace with proj tracking (has `.tracking` folder)
6. As an alternative, use `@proj` with your message - the auto-detection feature works in any mode

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
code --install-extension proj-1.7.28.vsix --force
```

Then restart VS Code.

---

## Version History

### 1.7.28

- Added comprehensive "Using @proj in Copilot Chat" guide covering slash commands, natural language, and automatic tools ranked by reliability
- Added "What to Expect" note setting honest expectations about @proj vs automatic logging
- Updated Quick Start Step 5 to honestly describe automatic logging as mode/model dependent

### 1.7.27

- Restored Copilot Chat integration for all UI actions (status bar menu and notification buttons open Copilot Chat)
- Added `@proj /end-auto` slash command for AI-generated session summaries
- Added auto-detection: `@proj` analyzes messages and automatically logs decisions, tasks, and blockers
- Fixed invisible confirmation dialog (changed from `showInputBox` to `showInformationMessage`)
- Fixed chained QuickPick issue with 150ms delay between UI elements
- Removed inline Language Model API code from extension.ts (delegates to `@proj /end-auto`)

### 1.7.21

- Added `@proj /end-auto` command for AI-generated summaries in Copilot Chat
- Fixed status bar "View Status" to show visible feedback
- Wrapped End Session in error handler for better error reporting

### 1.7.20

- Fixed CLI argument passing: switched from `exec` (shell string) to `execFile` (array args) so arguments with spaces work correctly

### 1.7.19

- Added progress notification during auto-summary generation
- Broadened Language Model API model search (removed `gpt-4` family filter)

### 1.7.18

- Added dedicated `proj-debug` output channel for VS Code debugging
- All extension log messages now visible in "proj-debug" output panel

### 1.7.0

- Version sync with CLI release
- Extension now published automatically to VS Code Marketplace via GitHub Actions

### 1.6.3

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
- Copilot can automatically log decisions, tasks, and blockers during conversation (Ask mode)
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
