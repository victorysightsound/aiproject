/**
 * proj VS Code Extension
 *
 * Integrates proj project tracking with VS Code and GitHub Copilot Chat.
 */

import * as vscode from 'vscode';
import * as cli from './cli';
import { registerParticipant } from './participant';
import * as statusBar from './statusBar';

/**
 * Extension activation
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    console.log('proj extension activating...');

    // Check if proj CLI is available
    const projInstalled = await cli.checkProjInstalled();
    if (!projInstalled) {
        vscode.window.showWarningMessage(
            'proj CLI not found. Please install it: brew install aiproject',
            'View Installation'
        ).then(selection => {
            if (selection === 'View Installation') {
                vscode.env.openExternal(
                    vscode.Uri.parse('https://github.com/victorysightsound/aiproject#installation')
                );
            }
        });
    }

    // Register chat participant (@proj)
    registerParticipant(context);

    // Create status bar
    statusBar.createStatusBar(context);

    // Register commands
    registerCommands(context);

    // Auto-run status on workspace open if tracking exists
    if (await cli.hasTracking()) {
        // Run status quietly to start/resume session
        cli.getStatus().catch(() => {
            // Ignore errors on auto-status
        });
    }

    console.log('proj extension activated');
}

/**
 * Register VS Code commands
 */
function registerCommands(context: vscode.ExtensionContext): void {
    // proj.status - Show status in output panel
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.status', async () => {
            const result = await cli.getResume();

            if (!result.success) {
                vscode.window.showErrorMessage(`proj error: ${result.stderr}`);
                return;
            }

            // Show in output channel
            const outputChannel = vscode.window.createOutputChannel('proj');
            outputChannel.clear();

            try {
                const data = JSON.parse(result.stdout);
                outputChannel.appendLine(`Project: ${data.project?.name || 'Unknown'}`);
                outputChannel.appendLine(`Session: #${data.session?.id || 'N/A'}`);
                outputChannel.appendLine('');

                if (data.last_session_summary) {
                    outputChannel.appendLine(`Last session: ${data.last_session_summary}`);
                    outputChannel.appendLine('');
                }

                if (data.blockers && data.blockers.length > 0) {
                    outputChannel.appendLine('BLOCKERS:');
                    for (const blocker of data.blockers) {
                        outputChannel.appendLine(`  - ${blocker.description}`);
                    }
                    outputChannel.appendLine('');
                }

                if (data.tasks && data.tasks.length > 0) {
                    outputChannel.appendLine('TASKS:');
                    for (const task of data.tasks) {
                        const icon = task.status === 'in_progress' ? '>' : '-';
                        outputChannel.appendLine(`  ${icon} [${task.priority}] ${task.description}`);
                    }
                    outputChannel.appendLine('');
                }

                if (data.decisions && data.decisions.length > 0) {
                    outputChannel.appendLine('RECENT DECISIONS:');
                    for (const decision of data.decisions.slice(0, 5)) {
                        outputChannel.appendLine(`  - ${decision.topic}: ${decision.decision}`);
                    }
                }
            } catch {
                outputChannel.appendLine(result.stdout);
            }

            outputChannel.show();
        })
    );

    // proj.tasks - Show tasks
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.tasks', async () => {
            const result = await cli.getTasks();

            if (!result.success) {
                vscode.window.showErrorMessage(`proj error: ${result.stderr}`);
                return;
            }

            const outputChannel = vscode.window.createOutputChannel('proj');
            outputChannel.clear();
            outputChannel.appendLine('TASKS:');
            outputChannel.appendLine('');
            outputChannel.appendLine(result.stdout || 'No active tasks.');
            outputChannel.show();
        })
    );

    // proj.endSession - End session with summary
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.endSession', async () => {
            const summary = await vscode.window.showInputBox({
                prompt: 'Session summary',
                placeHolder: 'What did you accomplish?'
            });

            if (!summary) {
                return;
            }

            const result = await cli.endSession(summary);

            if (result.success) {
                vscode.window.showInformationMessage(`Session ended: ${summary}`);
                statusBar.refresh();
            } else {
                vscode.window.showErrorMessage(`Failed to end session: ${result.stderr}`);
            }
        })
    );

    // proj.refresh - Refresh status bar
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.refresh', () => {
            statusBar.refresh();
            vscode.window.showInformationMessage('proj status refreshed');
        })
    );
}

/**
 * Extension deactivation
 */
export function deactivate(): void {
    statusBar.dispose();
    console.log('proj extension deactivated');
}
