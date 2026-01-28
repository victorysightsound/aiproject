/**
 * proj VS Code Extension
 *
 * Integrates proj project tracking with VS Code and GitHub Copilot Chat.
 */

import * as vscode from 'vscode';
import * as cli from './cli';
import { registerParticipant } from './participant';
import * as statusBar from './statusBar';
import { registerTools } from './tools';

// Debug output channel
let debugChannel: vscode.OutputChannel;

function log(message: string): void {
    if (!debugChannel) {
        debugChannel = vscode.window.createOutputChannel('proj-debug');
    }
    const timestamp = new Date().toISOString().substring(11, 23);
    debugChannel.appendLine(`[${timestamp}] ${message}`);
    console.log(`[proj] ${message}`);
}

/**
 * Extension activation
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    log('Extension activating...');

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

    // Register Language Model Tools for Copilot
    registerTools(context);

    // Create status bar
    statusBar.createStatusBar(context);

    // Register commands
    registerCommands(context);

    // Auto-status notification on workspace open if tracking exists
    if (await cli.hasTracking()) {
        showSessionNotification();
    }

    log('Extension activated');
}

/**
 * Register VS Code commands
 */
function registerCommands(context: vscode.ExtensionContext): void {
    // proj.status - Show status in output panel
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.status', async () => {
            log('proj.status command triggered');
            try {
                const result = await cli.getResume();
                log(`cli.getResume result: ${result.success}, ${result.stderr?.substring(0, 100)}`);

                if (!result.success) {
                    vscode.window.showErrorMessage(`proj error: ${result.stderr}`);
                    return;
                }

                // Show in output channel
                const outputChannel = vscode.window.createOutputChannel('proj');
                outputChannel.clear();

                try {
                    const data = JSON.parse(result.stdout);
                    const now = new Date().toLocaleTimeString();
                    outputChannel.appendLine(`=== proj status (${now}) ===`);
                    outputChannel.appendLine('');
                    outputChannel.appendLine(`Project: ${data.project?.name || 'Unknown'}`);
                    outputChannel.appendLine(`Session: #${data.current_session?.session_id || 'N/A'}`);
                    outputChannel.appendLine('');

                    if (data.last_session?.summary) {
                        outputChannel.appendLine(`Last session: ${data.last_session.summary}`);
                        outputChannel.appendLine('');
                    }

                    if (data.active_blockers && data.active_blockers.length > 0) {
                        outputChannel.appendLine('BLOCKERS:');
                        for (const blocker of data.active_blockers) {
                            outputChannel.appendLine(`  - ${blocker.description}`);
                        }
                        outputChannel.appendLine('');
                    }

                    if (data.active_tasks && data.active_tasks.length > 0) {
                        outputChannel.appendLine('TASKS:');
                        for (const task of data.active_tasks) {
                            const icon = task.status === 'in_progress' ? '>' : '-';
                            outputChannel.appendLine(`  ${icon} [${task.priority}] ${task.description}`);
                        }
                        outputChannel.appendLine('');
                    }

                    if (data.recent_decisions && data.recent_decisions.length > 0) {
                        outputChannel.appendLine('RECENT DECISIONS:');
                        for (const decision of data.recent_decisions.slice(0, 5)) {
                            outputChannel.appendLine(`  - ${decision.topic}: ${decision.decision}`);
                        }
                    }
                } catch (parseErr) {
                    log(`JSON parse error: ${parseErr}`);
                    outputChannel.appendLine(result.stdout);
                }

                outputChannel.show(false); // false = take focus
                vscode.window.showInformationMessage('proj status refreshed - see Output panel');
            } catch (err) {
                log(`proj.status error: ${err}`);
                vscode.window.showErrorMessage(`proj.status failed: ${err}`);
            }
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

    // proj.showMenu - Quick menu from status bar
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.showMenu', async () => {
            log('proj.showMenu command triggered');
            const choice = await vscode.window.showQuickPick([
                {
                    label: '$(info) View Status',
                    description: 'Show current project status',
                    value: 'status'
                },
                {
                    label: '$(checklist) View Tasks',
                    description: 'List all active tasks',
                    value: 'tasks'
                },
                {
                    label: '$(stop-circle) End Session...',
                    description: 'End session with summary',
                    value: 'end'
                },
                {
                    label: '$(refresh) Refresh',
                    description: 'Refresh status bar',
                    value: 'refresh'
                }
            ], {
                placeHolder: 'proj - Choose an action'
            });

            if (!choice) {
                return;
            }

            log(`Menu choice: ${choice.value}`);

            // Small delay to let the QuickPick fully close before opening another UI
            await new Promise(resolve => setTimeout(resolve, 150));

            switch (choice.value) {
                case 'status':
                    log('Opening Copilot Chat with @proj /status');
                    try {
                        await vscode.commands.executeCommand('workbench.action.chat.open', {
                            query: '@proj /status'
                        });
                    } catch {
                        await vscode.commands.executeCommand('proj.status');
                    }
                    break;
                case 'tasks':
                    log('Opening Copilot Chat with @proj /tasks');
                    try {
                        await vscode.commands.executeCommand('workbench.action.chat.open', {
                            query: '@proj /tasks'
                        });
                    } catch {
                        await vscode.commands.executeCommand('proj.tasks');
                    }
                    break;
                case 'end':
                    log('Opening Copilot Chat with @proj /end-auto');
                    try {
                        await vscode.commands.executeCommand('workbench.action.chat.open', {
                            query: '@proj /end-auto'
                        });
                    } catch {
                        await vscode.commands.executeCommand('proj.endSessionWithOptions');
                    }
                    break;
                case 'refresh':
                    log('Executing proj.refresh');
                    await vscode.commands.executeCommand('proj.refresh');
                    break;
            }
        })
    );

    // proj.endSessionWithOptions - End session with choice of manual or auto summary
    context.subscriptions.push(
        vscode.commands.registerCommand('proj.endSessionWithOptions', async () => {
            try {
            log('proj.endSessionWithOptions command triggered');

            // Use a notification dialog instead of QuickPick to avoid
            // chained-QuickPick issues when triggered from the status bar menu
            const choice = await vscode.window.showInformationMessage(
                'End session - how do you want to provide a summary?',
                { modal: false },
                'Enter Manually',
                'Auto-Generate'
            );
            log(`endSession choice: ${choice}`);

            if (!choice) {
                return;
            }

            if (choice === 'Enter Manually') {
                // Manual: prompt for summary
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
            } else {
                // Auto: open Copilot Chat with @proj /end-auto command
                log('Auto-summary selected - opening Copilot Chat');
                try {
                    await vscode.commands.executeCommand('workbench.action.chat.open', {
                        query: '@proj /end-auto'
                    });
                } catch {
                    // Fallback if chat command doesn't work
                    vscode.window.showInformationMessage(
                        'Open Copilot Chat and type: @proj /end-auto'
                    );
                }
            }
            } catch (err) {
                log(`endSessionWithOptions error: ${err}`);
                vscode.window.showErrorMessage(`End Session failed: ${err}`);
            }
        })
    );
}

/**
 * Show session notification on workspace open
 */
async function showSessionNotification(): Promise<void> {
    // Delay slightly to ensure VS Code window is fully ready
    await new Promise(resolve => setTimeout(resolve, 1500));

    try {
        // Get project status
        const result = cli.runProjSync(['resume', '--for-ai']);

        if (!result.success) {
            return; // Silently fail - don't bother user with errors on startup
        }

        const data = JSON.parse(result.stdout);

        // Build notification message
        const parts: string[] = [];

        // Session info
        if (data.current_session) {
            parts.push(`Session #${data.current_session.session_id}`);
        } else {
            parts.push('Session started');
        }

        // Task count
        const taskCount = data.active_tasks?.length || 0;
        if (taskCount > 0) {
            parts.push(`${taskCount} task${taskCount > 1 ? 's' : ''}`);
        }

        // Blocker count
        const blockerCount = data.active_blockers?.length || 0;
        if (blockerCount > 0) {
            parts.push(`${blockerCount} blocker${blockerCount > 1 ? 's' : ''}`);
        }

        // Last session summary
        const lastSummary = data.last_session?.summary;

        // Project name
        const projectName = data.project?.name || 'Project';

        // Show notification with more detail - use showWarningMessage for visibility
        // It stays until user clicks a button (doesn't auto-dismiss)
        let message = `proj: ${projectName} | ${parts.join(' | ')}`;
        if (lastSummary) {
            // Truncate if too long
            const truncatedSummary = lastSummary.length > 60
                ? lastSummary.substring(0, 57) + '...'
                : lastSummary;
            message += `\n\nLast: ${truncatedSummary}`;
        }

        // Use modal-style notification that requires user interaction
        const selection = await vscode.window.showInformationMessage(
            message,
            { modal: false },
            'View Full Status',
            'End Session',
            'OK'
        );

        if (selection === 'View Full Status') {
            try {
                await vscode.commands.executeCommand('workbench.action.chat.open', {
                    query: '@proj /status'
                });
            } catch {
                vscode.commands.executeCommand('proj.status');
            }
        } else if (selection === 'End Session') {
            try {
                await vscode.commands.executeCommand('workbench.action.chat.open', {
                    query: '@proj /end-auto'
                });
            } catch {
                vscode.commands.executeCommand('proj.endSessionWithOptions');
            }
        }

    } catch (error) {
        // Silently fail - don't bother user with parsing errors on startup
        log(`Failed to show session notification: ${error}`);
    }
}

/**
 * Extension deactivation
 */
export function deactivate(): void {
    statusBar.dispose();
    log('Extension deactivated');
}
