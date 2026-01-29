# Proj Efficiency Case Study: Prescribed Documentation Methods

## Comparing Structured Database Tracking vs. Embedded Code Comments

**Date:** January 29, 2026
**Version:** 1.0
**Tool Tested:** proj v1.8.2

---

## Executive Summary

This controlled test compared two prescribed documentation approaches for AI agents working across multiple sessions: structured database tracking (proj) vs. embedded code documentation (comments + git). Both approaches achieved identical accuracy metrics, but with different efficiency trade-offs.

### Key Findings

| Metric | With proj | Without proj | Difference |
|--------|-----------|--------------|-------------|
| Total files read | 12 | 36 | **67% reduction** |
| Estimated tokens | ~7,800 | ~10,500 | **26% reduction** |
| Commands run | 37 | 15 | 147% increase |
| Context recovery success | 100% | 100% | Equal |
| Decision consistency | 100% | 100% | Equal |
| Contradictions | 0 | 0 | Equal |

**Bottom line:** proj reduces token usage through more efficient context retrieval, but when both approaches include explicit documentation instructions, accuracy is equivalent. The test compared two structured documentation methods, not "tracking vs. nothing."

---

## 1. Introduction

### 1.1 Problem Statement

AI coding assistants operate without persistent memory between sessions. This creates challenges for multi-session projects:

1. **Context loss** - Previous decisions and rationale are forgotten
2. **Redundant reading** - Same files must be re-read each session
3. **Token waste** - Processing identical content repeatedly
4. **Inconsistency risk** - New sessions may contradict earlier decisions

### 1.2 Research Question

How do different documentation approaches compare for maintaining context across AI sessions?

This study compares:
- **Structured tracking:** proj database with `proj log decision`, `proj status`
- **Embedded documentation:** Code comments, TODO markers, git commits

### 1.3 Critical Design Note

**Both approaches were explicitly prescribed in the test prompts.** The "without proj" AI was instructed to use code comments and TODOs as the tracking mechanism. This makes the comparison fair (structured DB vs. embedded docs) but does not measure what happens when an AI has no tracking instructions at all.

---

## 2. Methodology

### 2.1 Test Design

**Controlled comparison:** Two identical Rust CLI bookmark manager projects.

| Project | Directory | Tracking Method |
|---------|-----------|-----------------|
| Test A | `~/projects/test-with-proj/` | proj database tracking |
| Test B | `~/projects/test-without-proj/` | Code comments + git |

**Test codebase:** A Rust CLI application with:
- 5 source files (cli.rs, commands.rs, main.rs, models.rs, storage.rs)
- Basic bookmark management functionality
- Several TODOs indicating work to be done

### 2.2 Session Structure

Four sessions were conducted on each project, each in a **fresh Claude conversation** with no prior memory:

| Session | Task |
|---------|------|
| 1 | Project understanding, make storage/search decisions, start HTML export |
| 2 | Context recovery test, continue previous work |
| 3 | Historical decision retrieval, plan new feature (tag hierarchy) |
| 4 | Final integration, complete pending work, code review |

Sessions were run in alternating order (A1, B1, A2, B2...) to reduce bias.

### 2.3 Prescribed Documentation Instructions

**With proj (Project A):**
```
Log this decision using: proj log decision "topic" "decision" "rationale"
Log blockers with: proj log blocker "description"
Create tasks with: proj task add "description" --priority high
```

**Without proj (Project B):**
```
Document your reasoning in a code comment or note.
Add a TODO comment in the relevant code.
```

**Both projects** were required to create metrics files at session end.

### 2.4 Metrics Collected

| Metric | Description |
|--------|-------------|
| Files read | Number of source files read by the agent |
| Commands run | Shell/tool commands executed |
| Estimated tokens | Based on file sizes and response lengths |
| Context recovery | Could the agent find previous decisions? |
| Contradictions | Did new work conflict with prior decisions? |
| Work duplication | Was any previous work unnecessarily redone? |

---

## 3. Test Results by Session

### 3.1 Session 1: Project Understanding and Initial Decisions

**Task:** Understand architecture, make storage/search decisions, start HTML export.

#### Results

| Metric | With proj | Without proj |
|--------|-----------|--------------|
| Files read | 7 | 9 |
| Commands run | 14 | 2 |
| Estimated tokens | ~2,500 | ~2,500 |

#### Key Observations

Both agents successfully:
- Understood the project architecture
- Made storage decisions (different choices - see below)
- Made search decisions
- Implemented HTML export
- Documented their work

**Critical Finding: Different Architectural Decisions**

Despite identical prompts, the agents made different choices:

| Decision | With proj | Without proj |
|----------|-----------|--------------|
| Storage | SQLite + FTS5 (planned migration) | Keep JSON (sufficient for now) |
| Search | Title-only (pending SQLite) | Title + URL + tags (implemented) |

This reflects AI non-determinism, not a proj vs. no-proj effect. However, the with-proj agent made more future-oriented decisions while without-proj was more pragmatic.

---

### 3.2 Session 2: Context Recovery

**Task:** Resume work from Session 1. Determine what was done and continue.

**Purpose:** Test ability to recover session context.

#### Results

| Metric | With proj | Without proj |
|--------|-----------|--------------|
| Files read | **1** | 5 |
| Commands run | 5 | 3 |
| Estimated tokens | ~1,500 | ~2,000 |
| Context recovery | Full | Full |

#### Context Recovery Methods

**With proj:**
1. Ran `proj status` (single command)
2. Immediately saw: 2 decisions, 5 tasks, session summary
3. Read 0 additional source files for context

**Without proj:**
1. Ran `git log` to find commit history
2. Ran `git show` for commit details
3. Read storage.rs to find decision comment (lines 5-9)
4. Read models.rs to find search decision comment (lines 63-65)
5. Read commands.rs to find TODOs

#### Accuracy Assessment

| Question | With proj | Without proj |
|----------|-----------|--------------|
| Contradict Session 1 decisions? | No | No |
| Duplicate Session 1 work? | No | No |
| Continue appropriately? | Yes | Yes |

Both achieved full context recovery, but proj required significantly fewer file reads.

---

### 3.3 Session 3: Historical Decision Retrieval

**Task:** Find and explain why the storage decision was made. Plan tag hierarchy feature.

**Purpose:** Test ability to retrieve past architectural decisions and check for conflicts.

#### Results

| Metric | With proj | Without proj |
|--------|-----------|--------------|
| Files read | 1 | 11 |
| Commands run | 8 | 5 |
| Estimated tokens | ~1,800 | ~3,500 |

#### Retrieval Methods

**With proj:**
```bash
proj context "storage" --ranked
```
Immediately returned decision with full rationale.

**Without proj:**
1. Read README.md for overview
2. Read metrics_session1.md (found reference to storage.rs:5-9)
3. Read metrics_session2.md
4. Read storage.rs to find actual decision comment
5. Read models.rs, commands.rs, cli.rs, main.rs
6. Multiple git log commands

#### Key Difference

The without-proj agent read **11 files** including previous metrics files to piece together the decision history. The proj agent used `proj context` to query directly.

**Bonus:** The proj agent also discovered related decisions (search-approach, html-export-safety) that connected to the storage decision.

---

### 3.4 Session 4: Final Integration

**Task:** Full context recovery, complete pending work (CSV export), code review.

**Purpose:** Test accumulated context value and final accuracy.

#### Results

| Metric | With proj | Without proj |
|--------|-----------|--------------|
| Files read | 3 | 11 |
| Commands run | 10 | 5 |
| Estimated tokens | ~2,000 | ~2,500 |

#### Final Context Recovery

**With proj:**
- Found 4 decisions: storage-format, search-approach, html-export-safety, tag-hierarchy
- Found 6 tasks (before session), added 2 more
- Found 0 blockers
- Identified incomplete work: CSV export, SQLite migration

**Without proj:**
- Found 4 decisions via metrics files and code comments
- Found scattered TODOs in code
- Identified incomplete work: CSV export

#### Work Completed

Both agents:
- Implemented CSV export with proper escaping
- Conducted code review
- Found no contradictions with previous decisions

---

## 4. Aggregate Analysis

### 4.1 Total Files Read

| Session | With proj | Without proj | Savings |
|---------|-----------|--------------|---------|
| 1: Understanding | 7 | 9 | 2 files |
| 2: Context Recovery | 1 | 5 | 4 files |
| 3: Historical Retrieval | 1 | 11 | 10 files |
| 4: Final Integration | 3 | 11 | 8 files |
| **Total** | **12** | **36** | **24 files (67%)** |

### 4.2 Commands Run

| Session | With proj | Without proj |
|---------|-----------|--------------|
| 1 | 14 | 2 |
| 2 | 5 | 3 |
| 3 | 8 | 5 |
| 4 | 10 | 5 |
| **Total** | **37** | **15** |

proj ran more commands (mostly proj CLI queries) but read far fewer files.

### 4.3 Token Estimation

**Calculation assumptions:**
- ~4 characters per token (code/text mix)
- proj command output: ~500-1000 chars per query
- Command overhead: ~50 tokens per command

**With proj:**

| Source | Est. Tokens |
|--------|-------------|
| Files read (12 files, partial) | ~4,500 |
| proj command outputs | ~2,000 |
| Command overhead (37 cmds) | ~1,850 |
| Agent prompts | ~600 |
| **Total Input** | **~8,950** |

**Without proj:**

| Source | Est. Tokens |
|--------|-------------|
| Files read (36 reads, many repeats) | ~9,000 |
| Git/command outputs | ~750 |
| Command overhead (15 cmds) | ~750 |
| Agent prompts | ~600 |
| **Total Input** | **~11,100** |

#### Estimated Savings

| Metric | With proj | Without proj | Difference |
|--------|-----------|--------------|------------|
| Input tokens | ~8,950 | ~11,100 | **-19%** |

**Note:** The without-proj agent compensated by using metrics files from previous sessions as a form of tracking, which reduced what would otherwise be larger differences.

### 4.4 Accuracy Comparison

| Metric | With proj | Without proj |
|--------|-----------|--------------|
| Decisions recovered | 4/4 (100%) | 4/4 (100%) |
| Contradictions | 0 | 0 |
| Work duplicated | 0 | 0 |
| Self-assessed accuracy | 5/5 | 5/5 |

**Both approaches achieved identical accuracy.** This is the key finding: when both approaches include explicit documentation instructions, accuracy is equivalent.

---

## 5. Implementation Comparison

### 5.1 Different Architectural Paths

The projects diverged architecturally:

| Aspect | With proj | Without proj |
|--------|-----------|--------------|
| Storage | Planned SQLite migration | Kept JSON |
| Search | Title-only | Title + URL + tags |
| HTML export | Flat list + escaping + TAGS attr | Tag-based folder grouping |
| Tags command | Not implemented | Implemented with hierarchy |

### 5.2 Code Completeness

| Feature | With proj | Without proj |
|---------|-----------|--------------|
| HTML entity escaping | Yes | No |
| TAGS attribute | Yes | No |
| Tag-based HTML folders | No | Yes |
| `tags` CLI command | No | Yes |
| `all_tags()` method | No | Yes |
| `leaf_tags()` method | No | Yes |
| CSV export | Yes | Yes |

**Lines of code:**
- With proj: models.rs (94 lines), commands.rs (167 lines)
- Without proj: models.rs (129 lines), commands.rs (199 lines)

**Finding:** Without-proj shipped more features despite higher token usage.

### 5.3 Decision Documentation Location

**With proj:**
- Decisions stored in `.tracking/tracking.db`
- Queryable via `proj context`, `proj status`
- Separated from code

**Without proj:**
- Decisions in code comments (storage.rs:5-9, models.rs:4-11, models.rs:114-116)
- Discoverable by reading source
- Self-documenting code

---

## 6. Qualitative Findings

### 6.1 Strengths of Proj Approach

1. **Single-command context recovery** - `proj status` provides complete history
2. **Queryable history** - `proj context "topic"` finds specific decisions
3. **Task management** - Centralized view of work items with priorities
4. **Separation of concerns** - Decisions tracked externally, code stays clean
5. **Scales with time** - Database grows but queries remain O(1)

### 6.2 Strengths of Code Comments Approach

1. **Self-documenting** - Decisions visible where implemented
2. **No external dependencies** - Works with any AI, any tooling
3. **Portable** - Anyone reading code sees decisions
4. **More features shipped** - Agent implemented more functionality
5. **Git as backup** - Commit messages provide secondary context

### 6.3 Why Without-Proj Shipped More Features

Possible explanations:

1. **More file reading = deeper familiarity** - Reading more code meant better understanding
2. **Decisions in code = more likely to implement** - Seeing the comment prompted action
3. **No task tracking = work on visible items** - Implemented what was in front of them vs. logged tasks
4. **Pragmatic decisions** - "Keep JSON" led to implementing current features rather than planning migrations

---

## 7. Limitations and Considerations

### 7.1 Test Limitations

1. **Small codebase** - 5 source files may not reveal scaling differences that emerge with larger projects

2. **Short timeline** - 4 sessions may not show long-term context decay

3. **Prescribed documentation** - Both approaches had explicit instructions. This does not measure what happens when an AI has no tracking guidance.

4. **AI non-determinism** - Identical prompts produced different architectural choices, complicating direct comparison

5. **Metrics files as compensation** - The without-proj agent used metrics files (required by test protocol) as a form of context tracking, reducing observed differences

### 7.2 What This Test Did NOT Measure

- What happens with no documentation instructions at all
- Whether AI spontaneously creates documentation
- Accuracy degradation over many sessions
- Performance with large codebases (100+ files)
- Multi-agent scenarios (different AI models)

### 7.3 Future Test Needed

A more revealing comparison would give AI agents **no tracking instructions** and measure:
- Does the AI spontaneously create documentation?
- How much context is lost between sessions?
- What accuracy degradation occurs?
- When does the AI contradict itself?

---

## 8. Conclusions

### 8.1 Primary Findings

1. **67% fewer files read** with proj across four sessions
2. **26% fewer estimated tokens** (less re-reading of source)
3. **Identical accuracy** - both approaches achieved 100% decision recovery
4. **Trade-off:** proj runs more commands; code comments produce more features

### 8.2 The Core Insight

When both approaches include explicit documentation instructions, **accuracy is equivalent**. The efficiency difference (fewer file reads, fewer tokens) is real but modest for small projects.

proj's primary value is not accuracy improvement over well-documented code - it's:
1. **Structured queryability** - Finding specific decisions without reading files
2. **Centralized task management** - Single view of all work items
3. **Scalability** - O(1) queries regardless of project history length
4. **Reduced cognitive load** - AI doesn't need to remember where comments are

### 8.3 Recommendations

**Use proj when:**
- Multi-session projects with frequent context switches
- Complex decision histories requiring queries
- Multiple AI agents or team members need shared context
- Long-running projects where history grows large
- You want structured task management

**Code comments may suffice when:**
- Small projects (< 10 source files)
- Single developer maintaining mental context
- Well-established documentation practices
- Simple decision histories
- You want decisions visible in source

**Hybrid approach (recommended):**
1. Use proj for structured tracking
2. Add brief code comments at implementation points
3. Reference proj decisions in comments for traceability

```rust
// Decision: storage-format (see proj log)
// Using JSON for simplicity; SQLite migration tracked as task #1
```

---

## Appendix A: Test Infrastructure

### A.1 Project Structure

```
bookmarks/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── cli.rs
    ├── models.rs
    ├── storage.rs
    └── commands.rs
```

### A.2 Session Prompts Summary

**Session 1 (both):**
- Understand project architecture
- Make storage decision (JSON vs SQLite)
- Make search decision
- Start HTML export implementation
- Document decisions (proj commands vs code comments)

**Session 2 (both):**
- Recover context from Session 1
- Continue HTML export work
- Respect previous decisions
- Handle any blockers

**Session 3 (both):**
- Find and explain storage decision rationale
- List all architectural decisions
- Design tag hierarchy feature
- Check for conflicts with existing decisions

**Session 4 (both):**
- Full context recovery
- Complete pending work (CSV export)
- Code review
- Verify no contradictions

### A.3 Metrics File Format (Required Both Projects)

```markdown
# Session N Metrics - WITH/WITHOUT PROJ

## Efficiency Metrics
- Files read: [count]
- Commands run: [count]
- Total response tokens (estimate): [count]

## Context Recovery
- Found previous decisions: Yes/No (list them)
- Found previous blockers: Yes/No
- Context recovery success: Full/Partial/Failed

## Accuracy Notes
- Did you contradict any previous decisions? Yes/No
- Did you duplicate any previous work? Yes/No

## Session Summary
[Brief description of what was accomplished]
```

---

## Appendix B: proj Tracking Data (Project A)

### B.1 Decisions Logged

| ID | Session | Topic | Decision |
|----|---------|-------|----------|
| 1 | 1 | storage-format | Use SQLite with rusqlite crate |
| 2 | 1 | search-approach | Use SQLite FTS5 full-text search |
| 3 | 2 | html-export-safety | Added HTML escaping and TAGS attribute |
| 4 | 3 | tag-hierarchy | Use path-based tags with / delimiter |

### B.2 Tasks Created

| ID | Description | Status | Priority |
|----|-------------|--------|----------|
| 1 | Migrate storage from JSON to SQLite | pending | high |
| 2 | Implement FTS5 search after migration | pending | high |
| 3 | Implement CSV export | completed | normal |
| 4 | Add description field to Bookmark | pending | normal |
| 5 | Implement browser bookmark import | pending | normal |
| 6 | Add CLI hierarchical tag display | pending | normal |
| 7 | Add edit command | pending | normal |
| 8 | Add open command (launch URL) | pending | normal |

### B.3 Session Summaries

| Session | Summary |
|---------|---------|
| 1 | Analyzed codebase; decided SQLite+FTS5; implemented HTML export; created 5 tasks |
| 2 | Recovered context via proj; enhanced HTML export with escaping and TAGS |
| 3 | Retrieved storage decision rationale; designed hierarchical tag matching |
| 4 | Full context recovery; implemented CSV export; code review; task cleanup |

---

## Appendix C: Code Comment Locations (Project B)

### C.1 Decision Comments

**storage.rs (lines 5-9):**
```rust
// DECISION: Stick with JSON for now
// Rationale: JSON is simple, portable, human-readable, and sufficient for personal use.
// SQLite would add a dependency and complexity. If the dataset grows large (1000+ bookmarks)
// or full-text search becomes critical, migrate to SQLite with FTS5.
// For now, simple contains-based search on JSON is adequate.
```

**models.rs (lines 4-11):**
```rust
// DECISION: Tag Hierarchy via Path Notation
// Rationale: Use "/" as separator (e.g., "programming/rust", "programming/python").
// - No schema change needed - tags remain Vec<String>
// - Backwards compatible with existing flat tags
// - Parent matching: filtering by "programming" includes all "programming/*" children
// - Aligns with JSON storage decision (no complex hierarchy structures)
// - Future: Could add tag taxonomy/aliases if users need shortcuts
```

**models.rs (lines 114-116):**
```rust
// DECISION: Use simple case-insensitive contains across title, URL, and tags
// Rationale: No new dependencies needed. Covers the most common search use cases.
// Future enhancement: fuzzy matching (fuzzy-matcher crate) or SQLite FTS5 if needed.
```

---

*End of Case Study*
