/**
 * CLI wrapper for executing proj commands
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import * as vscode from 'vscode';

const execAsync = promisify(exec);

export interface ProjResult {
    success: boolean;
    stdout: string;
    stderr: string;
}

/**
 * Get the configured path to the proj CLI
 */
function getProjPath(): string {
    const config = vscode.workspace.getConfiguration('proj');
    return config.get<string>('cliPath') || 'proj';
}

/**
 * Get the workspace folder path
 */
function getWorkspacePath(): string | undefined {
    const folders = vscode.workspace.workspaceFolders;
    if (folders && folders.length > 0) {
        return folders[0].uri.fsPath;
    }
    return undefined;
}

/**
 * Execute a proj command
 */
export async function runProj(args: string[]): Promise<ProjResult> {
    const projPath = getProjPath();
    const workspacePath = getWorkspacePath();

    if (!workspacePath) {
        return {
            success: false,
            stdout: '',
            stderr: 'No workspace folder open'
        };
    }

    const command = `${projPath} ${args.join(' ')}`;

    try {
        const { stdout, stderr } = await execAsync(command, {
            cwd: workspacePath,
            timeout: 30000 // 30 second timeout
        });

        return {
            success: true,
            stdout: stdout.trim(),
            stderr: stderr.trim()
        };
    } catch (error: any) {
        // exec throws on non-zero exit code
        return {
            success: false,
            stdout: error.stdout?.trim() || '',
            stderr: error.stderr?.trim() || error.message
        };
    }
}

/**
 * Check if proj is installed and accessible
 */
export async function checkProjInstalled(): Promise<boolean> {
    try {
        const result = await runProj(['--version']);
        return result.success;
    } catch {
        return false;
    }
}

/**
 * Check if the current workspace has proj tracking
 */
export async function hasTracking(): Promise<boolean> {
    const workspacePath = getWorkspacePath();
    if (!workspacePath) {
        return false;
    }

    const trackingPath = vscode.Uri.joinPath(
        vscode.Uri.file(workspacePath),
        '.tracking',
        'tracking.db'
    );

    try {
        await vscode.workspace.fs.stat(trackingPath);
        return true;
    } catch {
        return false;
    }
}

// Convenience functions for common commands

export async function getStatus(): Promise<ProjResult> {
    return runProj(['status', '--quiet']);
}

export async function getSnapshot(): Promise<ProjResult> {
    return runProj(['snapshot']);
}

export async function getResume(): Promise<ProjResult> {
    return runProj(['resume', '--for-ai']);
}

export async function getTasks(): Promise<ProjResult> {
    return runProj(['tasks']);
}

export async function endSession(summary: string): Promise<ProjResult> {
    return runProj(['session', 'end', `"${summary}"`]);
}

export async function logDecision(topic: string, decision: string, rationale?: string): Promise<ProjResult> {
    const args = ['log', 'decision', `"${topic}"`, `"${decision}"`];
    if (rationale) {
        args.push(`"${rationale}"`);
    }
    return runProj(args);
}

export async function logBlocker(description: string): Promise<ProjResult> {
    return runProj(['log', 'blocker', `"${description}"`]);
}

export async function logNote(category: string, title: string, content: string): Promise<ProjResult> {
    return runProj(['log', 'note', `"${category}"`, `"${title}"`, `"${content}"`]);
}

export async function addTask(description: string, priority?: string): Promise<ProjResult> {
    const args = ['task', 'add', `"${description}"`];
    if (priority) {
        args.push('--priority', priority);
    }
    return runProj(args);
}

export async function updateTask(id: number, status: string): Promise<ProjResult> {
    return runProj(['task', 'update', id.toString(), '--status', status]);
}

export async function searchContext(topic: string): Promise<ProjResult> {
    return runProj(['context', `"${topic}"`]);
}
