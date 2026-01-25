/**
 * Language Model Tools for proj integration with GitHub Copilot
 *
 * These tools allow Copilot to automatically call proj functions during conversation,
 * enabling logging of decisions, tasks, and blockers without explicit user commands.
 */

import * as vscode from 'vscode';
import * as cli from './cli';

/**
 * Tool result interface for Language Model Tools
 */
interface ToolResult {
    [key: string]: string | number | boolean | null;
}

/**
 * Tool: proj_get_status
 * Gets current project status including tasks, blockers, and recent decisions
 */
export class ProjGetStatusTool implements vscode.LanguageModelTool<{}> {
    async invoke(
        _options: vscode.LanguageModelToolInvocationOptions<{}>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const result = cli.runProjSync(['resume', '--for-ai']);

        if (!result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Error getting status: ${result.stderr}`)
            ]);
        }

        try {
            const data = JSON.parse(result.stdout);
            const summary = formatStatusSummary(data);
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(summary)
            ]);
        } catch {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(result.stdout)
            ]);
        }
    }
}

/**
 * Tool: proj_log_decision
 * Logs an architectural or design decision
 */
interface LogDecisionInput {
    topic: string;
    decision: string;
    rationale?: string;
}

export class ProjLogDecisionTool implements vscode.LanguageModelTool<LogDecisionInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<LogDecisionInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { topic, decision, rationale } = options.input;

        const args = ['log', 'decision', topic, decision];
        if (rationale) {
            args.push(rationale);
        }

        const result = cli.runProjSync(args);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(
                    `Decision logged: "${topic}" - ${decision}${rationale ? ` (${rationale})` : ''}`
                )
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to log decision: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_add_task
 * Adds a new task to the project
 */
interface AddTaskInput {
    description: string;
    priority?: 'high' | 'medium' | 'low';
}

export class ProjAddTaskTool implements vscode.LanguageModelTool<AddTaskInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<AddTaskInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { description, priority } = options.input;

        const args = ['task', 'add', description];
        if (priority) {
            args.push('--priority', priority);
        }

        const result = cli.runProjSync(args);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(
                    `Task added: "${description}"${priority ? ` [${priority}]` : ''}`
                )
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to add task: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_log_blocker
 * Logs a blocker that is preventing progress
 */
interface LogBlockerInput {
    description: string;
}

export class ProjLogBlockerTool implements vscode.LanguageModelTool<LogBlockerInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<LogBlockerInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { description } = options.input;

        const result = cli.runProjSync(['log', 'blocker', description]);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Blocker logged: "${description}"`)
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to log blocker: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_log_note
 * Logs a context note for future reference
 */
interface LogNoteInput {
    category: string;
    title: string;
    content: string;
}

export class ProjLogNoteTool implements vscode.LanguageModelTool<LogNoteInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<LogNoteInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { category, title, content } = options.input;

        const result = cli.runProjSync(['log', 'note', category, title, content]);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Note logged: [${category}] "${title}"`)
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to log note: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_update_task
 * Updates the status of an existing task
 */
interface UpdateTaskInput {
    taskId: number;
    status: 'pending' | 'in_progress' | 'completed' | 'blocked' | 'cancelled';
}

export class ProjUpdateTaskTool implements vscode.LanguageModelTool<UpdateTaskInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<UpdateTaskInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { taskId, status } = options.input;

        const result = cli.runProjSync(['task', 'update', taskId.toString(), '--status', status]);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Task #${taskId} updated to: ${status}`)
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to update task: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_end_session
 * Ends the current session with a summary
 */
interface EndSessionInput {
    summary: string;
}

export class ProjEndSessionTool implements vscode.LanguageModelTool<EndSessionInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<EndSessionInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { summary } = options.input;

        const result = cli.runProjSync(['session', 'end', summary]);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Session ended: "${summary}"`)
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to end session: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_search_context
 * Searches project context for a specific topic
 */
interface SearchContextInput {
    topic: string;
}

export class ProjSearchContextTool implements vscode.LanguageModelTool<SearchContextInput> {
    async invoke(
        options: vscode.LanguageModelToolInvocationOptions<SearchContextInput>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const { topic } = options.input;

        const result = cli.runProjSync(['context', topic]);

        if (result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(
                    result.stdout || `No context found for "${topic}"`
                )
            ]);
        } else {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Failed to search context: ${result.stderr}`)
            ]);
        }
    }
}

/**
 * Tool: proj_get_session_activity
 * Gets activity from the current session for generating end-of-session summaries
 */
export class ProjGetSessionActivityTool implements vscode.LanguageModelTool<{}> {
    async invoke(
        _options: vscode.LanguageModelToolInvocationOptions<{}>,
        _token: vscode.CancellationToken
    ): Promise<vscode.LanguageModelToolResult> {
        const result = cli.runProjSync(['resume', '--for-ai']);

        if (!result.success) {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(`Error getting session activity: ${result.stderr}`)
            ]);
        }

        try {
            const data = JSON.parse(result.stdout);
            const activity = formatSessionActivity(data);
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(activity)
            ]);
        } catch {
            return new vscode.LanguageModelToolResult([
                new vscode.LanguageModelTextPart(result.stdout)
            ]);
        }
    }
}

/**
 * Format session activity for summary generation
 */
function formatSessionActivity(data: any): string {
    const lines: string[] = [];

    lines.push('=== SESSION ACTIVITY FOR SUMMARY ===\n');

    // Session info
    if (data.current_session) {
        lines.push(`Session #${data.current_session.session_id}`);
        lines.push(`Started: ${data.current_session.started_at}`);
    }

    // Last session context
    if (data.last_session?.summary) {
        lines.push(`\nPrevious session: ${data.last_session.summary}`);
    }

    // Decisions made
    if (data.recent_decisions && data.recent_decisions.length > 0) {
        lines.push('\nDECISIONS MADE:');
        for (const decision of data.recent_decisions) {
            lines.push(`- ${decision.topic}: ${decision.decision}`);
            if (decision.rationale) {
                lines.push(`  Rationale: ${decision.rationale}`);
            }
        }
    }

    // Tasks
    if (data.active_tasks && data.active_tasks.length > 0) {
        const completed = data.active_tasks.filter((t: any) => t.status === 'completed');
        const inProgress = data.active_tasks.filter((t: any) => t.status === 'in_progress');
        const added = data.active_tasks.filter((t: any) => t.status === 'pending');

        if (completed.length > 0) {
            lines.push('\nTASKS COMPLETED:');
            for (const task of completed) {
                lines.push(`- ${task.description}`);
            }
        }

        if (inProgress.length > 0) {
            lines.push('\nTASKS IN PROGRESS:');
            for (const task of inProgress) {
                lines.push(`- ${task.description}`);
            }
        }

        if (added.length > 0) {
            lines.push('\nTASKS ADDED:');
            for (const task of added) {
                lines.push(`- ${task.description}`);
            }
        }
    }

    // Blockers
    if (data.active_blockers && data.active_blockers.length > 0) {
        lines.push('\nBLOCKERS:');
        for (const blocker of data.active_blockers) {
            lines.push(`- ${blocker.description}`);
        }
    }

    // Recent git commits if available
    if (data.recent_commits && data.recent_commits.length > 0) {
        lines.push('\nGIT COMMITS:');
        for (const commit of data.recent_commits) {
            lines.push(`- ${commit.message}`);
        }
    }

    lines.push('\n=== END SESSION ACTIVITY ===');
    lines.push('\nUse this information to generate a concise 1-2 sentence summary of what was accomplished.');

    return lines.join('\n');
}

/**
 * Register all Language Model Tools
 */
export function registerTools(context: vscode.ExtensionContext): void {
    // Register proj_get_status
    context.subscriptions.push(
        vscode.lm.registerTool('proj_get_status', new ProjGetStatusTool())
    );

    // Register proj_log_decision
    context.subscriptions.push(
        vscode.lm.registerTool('proj_log_decision', new ProjLogDecisionTool())
    );

    // Register proj_add_task
    context.subscriptions.push(
        vscode.lm.registerTool('proj_add_task', new ProjAddTaskTool())
    );

    // Register proj_log_blocker
    context.subscriptions.push(
        vscode.lm.registerTool('proj_log_blocker', new ProjLogBlockerTool())
    );

    // Register proj_log_note
    context.subscriptions.push(
        vscode.lm.registerTool('proj_log_note', new ProjLogNoteTool())
    );

    // Register proj_update_task
    context.subscriptions.push(
        vscode.lm.registerTool('proj_update_task', new ProjUpdateTaskTool())
    );

    // Register proj_end_session
    context.subscriptions.push(
        vscode.lm.registerTool('proj_end_session', new ProjEndSessionTool())
    );

    // Register proj_search_context
    context.subscriptions.push(
        vscode.lm.registerTool('proj_search_context', new ProjSearchContextTool())
    );

    // Register proj_get_session_activity
    context.subscriptions.push(
        vscode.lm.registerTool('proj_get_session_activity', new ProjGetSessionActivityTool())
    );

    console.log('[proj] Language Model Tools registered');
}

/**
 * Format status data into a human-readable summary
 */
function formatStatusSummary(data: any): string {
    const lines: string[] = [];

    // Project info
    if (data.project) {
        lines.push(`Project: ${data.project.name}`);
        if (data.project.description) {
            lines.push(`  ${data.project.description}`);
        }
    }

    // Session info
    if (data.current_session) {
        lines.push(`\nSession: #${data.current_session.session_id} (started ${data.current_session.started_at})`);
    }

    // Last session summary
    if (data.last_session?.summary) {
        lines.push(`\nLast session: ${data.last_session.summary}`);
    }

    // Blockers
    if (data.active_blockers && data.active_blockers.length > 0) {
        lines.push('\nBlockers:');
        for (const blocker of data.active_blockers) {
            lines.push(`  - ${blocker.description}`);
        }
    }

    // Tasks
    if (data.active_tasks && data.active_tasks.length > 0) {
        lines.push('\nTasks:');
        for (const task of data.active_tasks) {
            const status = task.status === 'in_progress' ? '>' : '-';
            lines.push(`  ${status} [${task.priority}] ${task.description}`);
        }
    }

    // Recent decisions
    if (data.recent_decisions && data.recent_decisions.length > 0) {
        lines.push('\nRecent Decisions:');
        for (const decision of data.recent_decisions.slice(0, 5)) {
            lines.push(`  - ${decision.topic}: ${decision.decision}`);
        }
    }

    return lines.join('\n');
}
