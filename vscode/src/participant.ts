/**
 * Chat participant for @proj integration with GitHub Copilot
 */

import * as vscode from 'vscode';
import * as cli from './cli';

const PARTICIPANT_ID = 'proj.assistant';

interface ProjChatResult extends vscode.ChatResult {
    metadata: {
        command: string;
    };
}

/**
 * Register the @proj chat participant
 */
export function registerParticipant(context: vscode.ExtensionContext): void {
    const participant = vscode.chat.createChatParticipant(
        PARTICIPANT_ID,
        handleChatRequest
    );

    participant.iconPath = new vscode.ThemeIcon('project');

    context.subscriptions.push(participant);
}

/**
 * Main handler for @proj chat requests
 */
async function handleChatRequest(
    request: vscode.ChatRequest,
    context: vscode.ChatContext,
    response: vscode.ChatResponseStream,
    token: vscode.CancellationToken
): Promise<ProjChatResult> {
    // Check if proj is available
    const hasProj = await cli.checkProjInstalled();
    if (!hasProj) {
        response.markdown(
            '**proj CLI not found.**\n\n' +
            'Please install proj first:\n\n' +
            '```bash\nbrew install aiproject\n```\n\n' +
            'Or see [installation instructions](https://github.com/victorysightsound/aiproject#installation).'
        );
        return { metadata: { command: 'error' } };
    }

    // Check if workspace has tracking
    const hasTracking = await cli.hasTracking();
    if (!hasTracking) {
        response.markdown(
            '**No proj tracking in this workspace.**\n\n' +
            'Initialize proj in your terminal:\n\n' +
            '```bash\nproj init\n```'
        );
        return { metadata: { command: 'error' } };
    }

    // Handle slash commands
    if (request.command) {
        return handleCommand(request.command, request.prompt, response, token);
    }

    // Handle natural language queries
    return handleQuery(request.prompt, response, token);
}

/**
 * Handle slash commands like /status, /tasks, etc.
 */
async function handleCommand(
    command: string,
    prompt: string,
    response: vscode.ChatResponseStream,
    token: vscode.CancellationToken
): Promise<ProjChatResult> {
    switch (command) {
        case 'status':
            return await handleStatusCommand(response);

        case 'tasks':
            return await handleTasksCommand(response);

        case 'decisions':
            return await handleDecisionsCommand(response);

        case 'log':
            return await handleLogCommand(prompt, response);

        case 'end':
            return await handleEndCommand(prompt, response);

        default:
            response.markdown(`Unknown command: /${command}`);
            return { metadata: { command: 'unknown' } };
    }
}

/**
 * Handle /status command
 */
async function handleStatusCommand(
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    response.progress('Getting project status...');

    const result = await cli.getResume();

    if (!result.success) {
        response.markdown(`**Error:** ${result.stderr}`);
        return { metadata: { command: 'status' } };
    }

    try {
        // Parse JSON output from --for-ai
        const data = JSON.parse(result.stdout);
        formatStatusResponse(data, response);
    } catch {
        // Fall back to raw output
        response.markdown('```\n' + result.stdout + '\n```');
    }

    return { metadata: { command: 'status' } };
}

/**
 * Handle /tasks command
 */
async function handleTasksCommand(
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    response.progress('Getting tasks...');

    const result = await cli.getTasks();

    if (!result.success) {
        response.markdown(`**Error:** ${result.stderr}`);
        return { metadata: { command: 'tasks' } };
    }

    if (!result.stdout || result.stdout.includes('(none)')) {
        response.markdown('No pending tasks.');
    } else {
        response.markdown('**Tasks:**\n\n```\n' + result.stdout + '\n```');
    }

    return { metadata: { command: 'tasks' } };
}

/**
 * Handle /decisions command
 */
async function handleDecisionsCommand(
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    response.progress('Getting recent decisions...');

    const result = await cli.getSnapshot();

    if (!result.success) {
        response.markdown(`**Error:** ${result.stderr}`);
        return { metadata: { command: 'decisions' } };
    }

    try {
        const data = JSON.parse(result.stdout);
        if (data.decisions && data.decisions.length > 0) {
            response.markdown('**Recent Decisions:**\n');
            for (const decision of data.decisions) {
                response.markdown(
                    `\n- **${decision.topic}**: ${decision.decision}` +
                    (decision.rationale ? ` _(${decision.rationale})_` : '')
                );
            }
        } else {
            response.markdown('No decisions logged yet.');
        }
    } catch {
        response.markdown('```\n' + result.stdout + '\n```');
    }

    return { metadata: { command: 'decisions' } };
}

/**
 * Handle /log command
 */
async function handleLogCommand(
    prompt: string,
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    // Parse the prompt to determine what to log
    const lowerPrompt = prompt.toLowerCase();

    if (lowerPrompt.startsWith('decision')) {
        // Try to parse: decision "topic" "decision" "rationale"
        const match = prompt.match(/decision\s+"([^"]+)"\s+"([^"]+)"(?:\s+"([^"]+)")?/i);
        if (match) {
            const result = await cli.logDecision(match[1], match[2], match[3]);
            if (result.success) {
                response.markdown(`Logged decision: **${match[1]}** - ${match[2]}`);
            } else {
                response.markdown(`**Error:** ${result.stderr}`);
            }
        } else {
            response.markdown(
                'Usage: `/log decision "topic" "decision" "rationale"`\n\n' +
                'Example: `/log decision "database" "Using SQLite" "Simple and portable"`'
            );
        }
    } else if (lowerPrompt.startsWith('blocker')) {
        const match = prompt.match(/blocker\s+"([^"]+)"/i);
        if (match) {
            const result = await cli.logBlocker(match[1]);
            if (result.success) {
                response.markdown(`Logged blocker: ${match[1]}`);
            } else {
                response.markdown(`**Error:** ${result.stderr}`);
            }
        } else {
            response.markdown(
                'Usage: `/log blocker "description"`\n\n' +
                'Example: `/log blocker "Waiting for API credentials"`'
            );
        }
    } else if (lowerPrompt.startsWith('note')) {
        const match = prompt.match(/note\s+"([^"]+)"\s+"([^"]+)"\s+"([^"]+)"/i);
        if (match) {
            const result = await cli.logNote(match[1], match[2], match[3]);
            if (result.success) {
                response.markdown(`Logged note: **${match[2]}**`);
            } else {
                response.markdown(`**Error:** ${result.stderr}`);
            }
        } else {
            response.markdown(
                'Usage: `/log note "category" "title" "content"`\n\n' +
                'Example: `/log note "constraint" "API limit" "Max 100 requests per minute"`'
            );
        }
    } else {
        response.markdown(
            '**Log types:**\n\n' +
            '- `/log decision "topic" "decision" "rationale"`\n' +
            '- `/log blocker "description"`\n' +
            '- `/log note "category" "title" "content"`'
        );
    }

    return { metadata: { command: 'log' } };
}

/**
 * Handle /end command
 */
async function handleEndCommand(
    prompt: string,
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    if (!prompt.trim()) {
        response.markdown(
            'Please provide a session summary:\n\n' +
            '`/end Implemented user authentication and fixed login bug`'
        );
        return { metadata: { command: 'end' } };
    }

    response.progress('Ending session...');

    const result = await cli.endSession(prompt.trim());

    if (result.success) {
        response.markdown(`Session ended with summary: "${prompt.trim()}"`);
    } else {
        response.markdown(`**Error:** ${result.stderr}`);
    }

    return { metadata: { command: 'end' } };
}

/**
 * Handle natural language queries
 */
async function handleQuery(
    prompt: string,
    response: vscode.ChatResponseStream,
    token: vscode.CancellationToken
): Promise<ProjChatResult> {
    const lowerPrompt = prompt.toLowerCase();

    // Route to appropriate handler based on keywords
    if (lowerPrompt.includes('status') || lowerPrompt.includes('where') || lowerPrompt.includes('left off')) {
        return handleStatusCommand(response);
    }

    if (lowerPrompt.includes('task') || lowerPrompt.includes('todo') || lowerPrompt.includes('pending')) {
        return handleTasksCommand(response);
    }

    if (lowerPrompt.includes('decision') || lowerPrompt.includes('decided') || lowerPrompt.includes('why did')) {
        return handleDecisionsCommand(response);
    }

    if (lowerPrompt.includes('block') || lowerPrompt.includes('stuck') || lowerPrompt.includes('waiting')) {
        return await handleBlockersQuery(response);
    }

    // Search for topic
    if (lowerPrompt.includes('about') || lowerPrompt.includes('context')) {
        const topic = extractTopic(prompt);
        if (topic) {
            return await handleSearchQuery(topic, response);
        }
    }

    // Default: show full context
    response.progress('Getting project context...');
    const result = await cli.getResume();

    if (result.success) {
        try {
            const data = JSON.parse(result.stdout);
            formatStatusResponse(data, response);
        } catch {
            response.markdown('```\n' + result.stdout + '\n```');
        }
    } else {
        response.markdown(`**Error:** ${result.stderr}`);
    }

    return { metadata: { command: 'query' } };
}

/**
 * Handle blockers query
 */
async function handleBlockersQuery(
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    response.progress('Checking for blockers...');

    const result = await cli.getSnapshot();

    if (!result.success) {
        response.markdown(`**Error:** ${result.stderr}`);
        return { metadata: { command: 'blockers' } };
    }

    try {
        const data = JSON.parse(result.stdout);
        if (data.blockers && data.blockers.length > 0) {
            response.markdown('**Current Blockers:**\n');
            for (const blocker of data.blockers) {
                response.markdown(`\n- ${blocker.description}`);
            }
        } else {
            response.markdown('No active blockers.');
        }
    } catch {
        response.markdown('Could not parse blockers.');
    }

    return { metadata: { command: 'blockers' } };
}

/**
 * Handle search query
 */
async function handleSearchQuery(
    topic: string,
    response: vscode.ChatResponseStream
): Promise<ProjChatResult> {
    response.progress(`Searching for "${topic}"...`);

    const result = await cli.searchContext(topic);

    if (result.success) {
        if (result.stdout) {
            response.markdown(`**Context for "${topic}":**\n\n` + result.stdout);
        } else {
            response.markdown(`No results found for "${topic}".`);
        }
    } else {
        response.markdown(`**Error:** ${result.stderr}`);
    }

    return { metadata: { command: 'search' } };
}

/**
 * Format status response from JSON data
 */
function formatStatusResponse(data: any, response: vscode.ChatResponseStream): void {
    // Project info
    if (data.project) {
        response.markdown(`## ${data.project.name}\n`);
        if (data.project.description) {
            response.markdown(`_${data.project.description}_\n`);
        }
    }

    // Session info
    if (data.session) {
        response.markdown(`\n**Session #${data.session.id}** started ${data.session.started_at}\n`);
    }

    // Last session summary
    if (data.last_session_summary) {
        response.markdown(`\n**Last session:** ${data.last_session_summary}\n`);
    }

    // Blockers
    if (data.blockers && data.blockers.length > 0) {
        response.markdown('\n### Blockers\n');
        for (const blocker of data.blockers) {
            response.markdown(`- ${blocker.description}\n`);
        }
    }

    // Tasks
    if (data.tasks && data.tasks.length > 0) {
        response.markdown('\n### Tasks\n');
        for (const task of data.tasks) {
            const icon = task.status === 'in_progress' ? '🔄' : '⭕';
            response.markdown(`${icon} [${task.priority}] ${task.description}\n`);
        }
    }

    // Recent decisions
    if (data.decisions && data.decisions.length > 0) {
        response.markdown('\n### Recent Decisions\n');
        for (const decision of data.decisions.slice(0, 5)) {
            response.markdown(`- **${decision.topic}**: ${decision.decision}\n`);
        }
    }
}

/**
 * Extract a topic from a natural language query
 */
function extractTopic(prompt: string): string | null {
    // Try to extract topic after "about"
    const aboutMatch = prompt.match(/about\s+["']?([^"'?]+)["']?/i);
    if (aboutMatch) {
        return aboutMatch[1].trim();
    }

    // Try to extract topic after "context"
    const contextMatch = prompt.match(/context\s+(?:for\s+)?["']?([^"'?]+)["']?/i);
    if (contextMatch) {
        return contextMatch[1].trim();
    }

    return null;
}
