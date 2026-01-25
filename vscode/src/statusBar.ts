/**
 * Status bar integration for proj
 */

import * as vscode from 'vscode';
import * as cli from './cli';

let statusBarItem: vscode.StatusBarItem | undefined;
let refreshInterval: NodeJS.Timeout | undefined;

/**
 * Create and show the status bar item
 */
export function createStatusBar(context: vscode.ExtensionContext): void {
    const config = vscode.workspace.getConfiguration('proj');
    if (!config.get<boolean>('showStatusBar')) {
        return;
    }

    statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Left,
        100
    );

    statusBarItem.command = 'proj.showMenu';
    statusBarItem.tooltip = 'Click for proj options';

    context.subscriptions.push(statusBarItem);

    // Initial update
    updateStatusBar();

    // Refresh every 60 seconds
    refreshInterval = setInterval(updateStatusBar, 60000);

    context.subscriptions.push({
        dispose: () => {
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }
        }
    });
}

/**
 * Update the status bar with current project state
 */
export async function updateStatusBar(): Promise<void> {
    if (!statusBarItem) {
        return;
    }

    // Check if workspace has tracking
    const hasTracking = await cli.hasTracking();
    if (!hasTracking) {
        statusBarItem.hide();
        return;
    }

    try {
        const result = await cli.getSnapshot();

        if (!result.success) {
            statusBarItem.text = '$(warning) proj';
            statusBarItem.tooltip = 'proj: Error getting status';
            statusBarItem.show();
            return;
        }

        const data = JSON.parse(result.stdout);

        // Build status text
        let text = '$(project) proj';
        const parts: string[] = [];

        // Add session indicator
        if (data.session) {
            parts.push(`#${data.session.id}`);
        }

        // Add task count
        if (data.tasks && data.tasks.length > 0) {
            const pendingCount = data.tasks.filter((t: any) => t.status === 'pending').length;
            const inProgressCount = data.tasks.filter((t: any) => t.status === 'in_progress').length;

            if (inProgressCount > 0) {
                parts.push(`${inProgressCount} active`);
            } else if (pendingCount > 0) {
                parts.push(`${pendingCount} tasks`);
            }
        }

        // Add blocker indicator
        if (data.blockers && data.blockers.length > 0) {
            parts.push('$(warning) blocked');
        }

        if (parts.length > 0) {
            text += ` (${parts.join(', ')})`;
        }

        statusBarItem.text = text;

        // Build tooltip
        let tooltip = `**proj** - ${data.project?.name || 'Unknown'}\n\n`;

        if (data.session) {
            tooltip += `Session #${data.session.id}\n`;
        }

        if (data.tasks && data.tasks.length > 0) {
            tooltip += `\nTasks: ${data.tasks.length} active\n`;
        }

        if (data.blockers && data.blockers.length > 0) {
            tooltip += `\nBlockers: ${data.blockers.length}\n`;
        }

        tooltip += '\nClick to show full status';

        statusBarItem.tooltip = new vscode.MarkdownString(tooltip);
        statusBarItem.show();

    } catch (error) {
        statusBarItem.text = '$(project) proj';
        statusBarItem.tooltip = 'proj: Click to show status';
        statusBarItem.show();
    }
}

/**
 * Force refresh the status bar
 */
export function refresh(): void {
    updateStatusBar();
}

/**
 * Hide and dispose the status bar
 */
export function dispose(): void {
    if (refreshInterval) {
        clearInterval(refreshInterval);
    }
    if (statusBarItem) {
        statusBarItem.dispose();
    }
}
