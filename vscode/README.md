# proj VS Code Extension

Integrates [proj](https://github.com/victorysightsound/aiproject) project tracking with VS Code and GitHub Copilot Chat.

## Features

### @proj Chat Participant

Query your project context directly in Copilot Chat:

```
@proj what's my current status?
@proj show my tasks
@proj what did we decide about authentication?
@proj /status
@proj /tasks
@proj /end Implemented user authentication
```

### Slash Commands

| Command | Description |
|---------|-------------|
| `/status` | Show current project status |
| `/tasks` | List pending tasks |
| `/decisions` | Show recent decisions |
| `/log` | Log a decision, note, or blocker |
| `/end` | End the current session with a summary |

### Logging Examples

```
@proj /log decision "database" "Using SQLite" "Simple and portable"
@proj /log blocker "Waiting for API credentials"
@proj /log note "constraint" "API limit" "Max 100 requests per minute"
```

### Status Bar

Shows current session and task count. Click to view full status.

### Commands (Cmd/Ctrl+Shift+P)

- `proj: Show Status` - Display project status
- `proj: List Tasks` - Show all tasks
- `proj: End Session` - End session with summary
- `proj: Refresh` - Refresh status bar

## Requirements

- [proj CLI](https://github.com/victorysightsound/aiproject) must be installed
- Workspace must have proj tracking initialized (`proj init`)

## Installation

### From VSIX (Local)

```bash
cd vscode
npm install
npm run compile
npm run package
code --install-extension proj-1.4.0.vsix
```

### From Marketplace

Coming soon.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `proj.cliPath` | `proj` | Path to proj CLI executable |
| `proj.showStatusBar` | `true` | Show status bar indicator |

## Development

```bash
cd vscode
npm install
npm run watch
```

Press F5 in VS Code to launch Extension Development Host.

## License

MIT
