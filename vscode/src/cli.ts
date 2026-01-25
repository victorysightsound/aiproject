/**
 * CLI wrapper for executing proj commands
 */

import { exec, execSync } from 'child_process';
import { promisify } from 'util';
import * as vscode from 'vscode';

const execAsync = promisify(exec);

/**
 * Execute a proj command synchronously (for debugging)
 */
export function runProjSync(args: string[]): ProjResult {
    const projPath = getProjPath();
    const workspacePath = getWorkspacePath();

    if (!workspacePath) {
        return {
            success: false,
            stdout: '',
            stderr: 'No workspace folder open'
        };
    }

    // Use full path to proj to avoid PATH issues
    const fullProjPath = '/usr/local/bin/proj';
    const command = `${fullProjPath} ${args.join(' ')}`;
    console.log(`[proj] Running sync: ${command} in ${workspacePath}`);

    try {
        const stdout = execSync(command, {
            cwd: workspacePath,
            timeout: 10000,
            encoding: 'utf8',
            stdio: ['pipe', 'pipe', 'pipe'], // Explicit stdio
            env: {
                ...process.env,
                NO_COLOR: '1',
                PATH: '/usr/local/bin:/usr/bin:/bin'
            }
        });

        console.log(`[proj] Sync success, length: ${stdout?.length}`);
        return {
            success: true,
            stdout: stdout.trim(),
            stderr: ''
        };
    } catch (error: any) {
        console.log(`[proj] Sync error: ${error.message}`);
        return {
            success: false,
            stdout: error.stdout?.toString().trim() || '',
            stderr: error.stderr?.toString().trim() || error.message
        };
    }
}

export interface ProjResult {
    success: boolean;
    stdout: string;
    stderr: string;
}

/**
 * Get the configured path to the proj CLI
 * Defaults to full path since VS Code extension host may not have /usr/local/bin in PATH
 */
function getProjPath(): string {
    const config = vscode.workspace.getConfiguration('proj');
    return config.get<string>('cliPath') || '/usr/local/bin/proj';
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
    console.log(`[proj] Running: ${command} in ${workspacePath}`);

    try {
        const { stdout, stderr } = await execAsync(command, {
            cwd: workspacePath,
            timeout: 30000, // 30 second timeout
            env: { ...process.env, NO_COLOR: '1' } // Disable colors for clean output
        });

        console.log(`[proj] Success: ${stdout.substring(0, 100)}...`);
        return {
            success: true,
            stdout: stdout.trim(),
            stderr: stderr.trim()
        };
    } catch (error: any) {
        console.log(`[proj] Error: ${error.message}`);
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
