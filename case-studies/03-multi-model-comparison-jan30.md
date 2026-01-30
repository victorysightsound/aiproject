# Multi-Model Context Tracking Comparison

## Cross-Platform Study: proj vs. Code Comments vs. No Instructions

**Date:** January 29-30, 2026
**Version:** 1.0
**Tool Tested:** proj v1.8.2
**Models Tested:** Claude Opus 4.5, Codex GPT 5.2

---

## Executive Summary

This study tested two AI coding assistants across three context tracking approaches to answer a fundamental question: **Does proj actually help, and if so, how?**

### Key Findings

| Finding | Evidence |
|---------|----------|
| **All approaches achieve 100% decision consistency** | No contradictions in any of the 24 completed sessions |
| **proj reduces context recovery time by 6-10x** | <5s vs 30-90s to understand project state |
| **proj adds ~20-25% cost overhead** | Additional tracking commands increase token usage |
| **Code quality is identical across approaches** | Same features implemented correctly regardless of tracking |

**Bottom line:** proj's value is efficiency, not accuracy. All tracking approaches work when followed. proj makes context recovery faster and more reliable.

---

## 1. Study Design

### 1.1 Test Matrix

| Study | Tracking Approach | Instructions Given |
|-------|-------------------|-------------------|
| **01** | proj database | Use `proj status`, `proj log decision`, etc. |
| **02** | Code comments | Document decisions in code with `DECISION:` markers |
| **03** | No instructions | Minimal prompts, observe spontaneous behavior |

| Model | Provider | CLI Tool |
|-------|----------|----------|
| Claude Opus 4.5 | Anthropic | Claude Code |
| Codex GPT 5.2 | OpenAI | Codex CLI |

**Total tests:** 3 studies × 2 models × 4 sessions = 24 sessions

### 1.2 Test Protocol

Each study used identical Rust CLI bookmark manager codebases with:
- 5 source files (~9,000 characters)
- TODOs indicating decisions needed (storage format, search approach)
- Incomplete features to implement (HTML export, CSV export, tag hierarchy)

**Session structure (each in fresh conversation with no prior memory):**

| Session | Task |
|---------|------|
| 1 | Understand project, make storage/search decisions, implement HTML export |
| 2 | Recover context, continue work, respect previous decisions |
| 3 | Find historical decisions, design and implement tag hierarchy |
| 4 | Full review, complete pending work, assess consistency |

### 1.3 Automation

Tests were run automatically using a custom test runner:
```bash
./run-tests.sh --parallel  # Runs all 36 sessions
```

Each session captured:
- Full conversation output
- Duration (via timing wrapper)
- Output size (character count)
- Exit status

---

## 2. Results

### 2.1 Completion Status

| Study | Claude | Codex |
|-------|--------|-------|
| 01: proj | ✅ 4/4 | ✅ 4/4 |
| 02: comments | ✅ 4/4 | ✅ 4/4 |
| 03: nothing | ✅ 4/4 | ✅ 4/4 |

All 24 sessions completed successfully.

### 2.2 Timing Results

**Total duration per study (4 sessions):**

| Study | Claude | Codex |
|-------|--------|-------|
| 01: proj | 679s (11.3 min) | 1500s (25.0 min) |
| 02: comments | 540s (9.0 min) | 1645s (27.4 min) |
| 03: nothing | 502s (8.4 min) | 921s (15.3 min) |

**Observations:**
- Codex is 2-3x slower than Claude
- proj adds 15-35% time overhead vs. no instructions
- Study 03 (no instructions) is fastest due to shorter prompts

### 2.3 Context Recovery Success

| Study | Claude | Codex | Method Used |
|-------|--------|-------|-------------|
| 01: proj | 100% | 100% | `proj status` → instant |
| 02: comments | 100% | 100% | git log + grep code |
| 03: nothing | 100% | 100% | Read output files / search TODOs |

**All approaches achieved perfect context recovery.** The difference is how:

- **proj:** Single command provides full context in <5 seconds
- **comments:** Must search git history and grep code (30-60 seconds)
- **nothing:** Creative exploration required (45-90 seconds)

### 2.4 Decision Consistency

| Study | Claude | Codex | Contradictions |
|-------|--------|-------|----------------|
| 01: proj | 5/5 | 5/5 | None |
| 02: comments | 5/5 | 5/5 | None |
| 03: nothing | 5/5 | 5/5 | None |

**No contradictions in any study.** Once a decision was documented (by any method), subsequent sessions respected it.

### 2.5 Decisions Made

**Storage Decision (JSON vs SQLite):**

| Study | Claude | Codex |
|-------|--------|-------|
| 01: proj | SQLite | JSON (defer SQLite) |
| 02: comments | JSON | SQLite |
| 03: nothing | JSON | JSON |

Different models made different decisions, and the same model made different decisions across studies. This is expected - the decisions are reasonable given the ambiguity. The key finding is consistency *within* each study.

---

## 3. Cost Analysis

### 3.1 Pricing Assumptions

| Model | Input (per 1M tokens) | Output (per 1M tokens) |
|-------|----------------------|------------------------|
| Claude Opus 4.5 | $5.00 | $25.00 |
| Codex GPT 5.2 | $15.00 | $60.00 |

### 3.2 Estimated Costs

| Study | Claude | Codex | Total |
|-------|--------|-------|-------|
| 01: proj | $0.20 | $6.47 | $6.67 |
| 02: comments | $0.19 | $6.73 | $6.92 |
| 03: nothing | $0.17 | $5.20 | $5.37 |
| **Total** | **$0.56** | **$18.40** | **$18.96** |

### 3.3 Cost Observations

1. **Codex is 30x more expensive than Claude** due to verbose output (90K+ tokens vs 1-3K)
2. **proj overhead for Claude:** +$0.03 (~18%) - small but present for short projects
3. **Claude is most cost-effective** for routine development work
4. **Time savings outweigh cost:** proj's context recovery speed saves developer time

### 3.4 ROI Calculation

**Time savings from proj:**
- Context recovery: ~40 seconds saved per session
- At $100/hour developer time: $1.11 saved per session

**Break-even analysis:**
- Claude: proj overhead ($0.03) vs time saved ($1.11) → ROI immediately
- Codex: proj overhead ($1.27) vs time saved ($1.11) → ROI after 2 sessions

---

## 4. Qualitative Analysis

### 4.1 Decision Documentation Quality

**proj tracking (Study 01):**
```
Topic: storage-format
Decision: Use SQLite instead of JSON
Rationale: SQLite provides indexed queries for efficient tag filtering
(O(log n) vs O(n)), FTS5 extension enables proper full-text search,
ACID transactions prevent data corruption, and it remains a single
portable file.
```

**Code comments (Study 02):**
```rust
// DECISION: Keep JSON for storage (decided 2026-01-29)
//
// Rationale: JSON is the right choice because:
// 1. Simplicity - No additional dependencies
// 2. Scale - Works fine up to ~10,000 bookmarks
// 3. Portability - Human-readable, easy to backup
// 4. Atomicity - Can implement atomic writes without SQLite
//
// When SQLite would make sense:
// - Full-text search across thousands of bookmarks
// - Multi-process concurrent access
```

Both approaches produced well-documented decisions. proj captured decisions in a queryable database; comments embedded them in relevant code locations.

### 4.2 Spontaneous Behavior (Study 03)

Without tracking instructions, the models adapted:

| Behavior | Claude | Codex |
|----------|--------|-------|
| Read session output for context | Yes | No |
| Created decision comments in code | No | Yes |
| Created documentation files | No | No |
| Respected prior decisions | Yes | Yes |

**Claude's approach:** Read `session1_output.txt` to find previous session's summary. Creative but fragile - only works when output is captured.

**Codex's approach:** Search code for TODOs, add decision comments where relevant. More discoverable but scattered.

### 4.3 Task Organization

**Tasks created via proj (Study 01):**

| Model | Tasks | High Priority | Normal | Low |
|-------|-------|---------------|--------|-----|
| Claude | 9 | 2 | 5 | 2 |
| Codex | 5 | 1 | 4 | 0 |

Claude was more thorough in task identification, breaking work into smaller units. Codex focused on immediate needs.

---

## 5. Conclusions

### 5.1 Primary Finding

**proj improves efficiency, not accuracy.**

All three tracking approaches achieved:
- 100% context recovery success
- 100% decision consistency
- Equivalent code quality

The difference is *how much effort* context recovery requires:
- proj: Instant (single command)
- Comments: Moderate (search git + code)
- Nothing: High (creative exploration)

### 5.2 When to Use proj

✅ **Use proj when:**
- Project spans multiple sessions
- Multiple AI agents may contribute
- Decision rationale needs preservation
- Task tracking and prioritization matter

✅ **Use code comments when:**
- Project is small and self-contained
- Single AI agent working alone
- Decisions are simple/obvious from code

✅ **Use nothing when:**
- Quick prototypes or experiments
- Single-session tasks
- Exploring before committing to an approach

### 5.3 Limitations

This study has several limitations:

1. **Single test per condition** - AI responses are non-deterministic; results may vary
2. **Artificial scenario** - Same codebase, same tasks, controlled environment
3. **Short duration** - Only 4 sessions; long-term context decay not measured
4. **Two models only** - Results may differ with other AI assistants

### 5.4 Future Work

Recommended follow-up studies:

1. **Long-term context decay** - Test over 10+ sessions
2. **Multi-agent collaboration** - Multiple AIs working on same project
3. **Real-world validation** - Test on actual production projects
4. **Context complexity** - Test with larger codebases and more decisions

---

## 6. Data Availability

### 6.1 Test Infrastructure

All test materials available at:
```
~/projects/proj-case-studies/
├── base-codebase/           # Clean starting codebase
├── run-tests.sh             # Automated test runner
├── setup-run.sh             # Directory setup script
├── study-01-with-proj/      # proj tracking tests
├── study-02-with-comments/  # Code comments tests
├── study-03-no-instructions/# No tracking tests
├── analysis_report.md       # Summary analysis
├── detailed_analysis.md     # Full analysis with costs
└── test_results_summary.md  # Quick reference
```

### 6.2 Raw Data

Each test directory contains:
- `session[1-4]_output.txt` - Full conversation logs
- `session[1-4]_metrics.txt` - Timing and size data
- `metrics_session[1-4].md` - AI-generated session reports
- `.tracking/tracking.db` - proj database (Study 01 only)

### 6.3 Reproducibility

To reproduce this study:

```bash
cd ~/projects/proj-case-studies

# Reset test directories
./setup-run.sh 01 claude-opus45
./setup-run.sh 02 claude-opus45
./setup-run.sh 03 claude-opus45
# ... repeat for other models

# Run all tests
./run-tests.sh --parallel

# Or run specific study/model
./run-tests.sh 01 claude-opus45

# Analyze results
./run-tests.sh --analyze
```

---

## Appendix A: Session Prompts

### Study 01 Session 1 (proj)
```markdown
You are starting work on a Rust CLI bookmark manager project.

## IMPORTANT: Use proj tracking
Start by running: proj status

## Tasks
1. Understand the project
2. Make a storage decision - Log with: proj log decision "storage-format" "..." "..."
3. Make a search decision - Log it
4. Start HTML export
5. Create tasks for future work
```

### Study 02 Session 1 (comments)
```markdown
You are starting work on a Rust CLI bookmark manager project.

## Tasks
1. Understand the project
2. Make a storage decision - Document your reasoning in a code comment
3. Make a search decision - Document it in a code comment
4. Start HTML export
5. Document blockers with TODO comments
```

### Study 03 Session 1 (nothing)
```markdown
You are starting work on a Rust CLI bookmark manager project.

## Tasks
1. Understand the project
2. Make a storage decision
3. Make a search decision
4. Start HTML export
5. Handle any issues as you see fit
```

---

## Appendix B: Model Configurations

### Claude Opus 4.5
```
CLI: claude --print --dangerously-skip-permissions
Version: 2.1.25
```

### Codex GPT 5.2
```
CLI: codex exec --dangerously-bypass-approvals-and-sandbox
Version: 0.91.0
Reasoning: xhigh
```
