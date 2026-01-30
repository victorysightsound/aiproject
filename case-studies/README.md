# Proj Case Studies

Controlled tests measuring proj's effectiveness for AI-assisted development.

## Completed Studies

### 01. Inventory CLI (January 24, 2026)

**Test:** proj tracking vs. no tracking at all

**Finding:** proj reduces token usage by ~50% and enables context recovery that is impossible without any tracking.

- [Full Case Study (Markdown)](01-inventory-cli-jan24.md)
- [Full Case Study (PDF)](01-inventory-cli-jan24.pdf)

| Metric | With proj | Without proj | Improvement |
|--------|-----------|--------------|-------------|
| Files read | 11 | 34 | 68% reduction |
| Token usage | ~10K | ~20K | 50% reduction |
| Context recovery | 100% | 0% | Critical |

### 02. Bookmarks CLI (January 29, 2026)

**Test:** proj database tracking vs. prescribed code comments

**Finding:** Both approaches achieve identical accuracy when documentation is required. proj reduces file reads by 67% through more efficient context retrieval.

- [Full Case Study (Markdown)](02-bookmarks-cli-jan29.md)

| Metric | With proj | Without proj | Difference |
|--------|-----------|--------------|-------------|
| Files read | 12 | 36 | 67% reduction |
| Estimated tokens | ~7,800 | ~10,500 | 26% reduction |
| Context recovery | 100% | 100% | Equal |
| Accuracy | 100% | 100% | Equal |

**Important context:** The "without proj" AI was explicitly instructed to use code comments as the tracking alternative. This tests two documentation methods, not tracking vs. nothing.

### 03. Multi-Model Comparison (January 29-30, 2026)

**Test:** proj vs. code comments vs. no instructions, across Claude and Codex

**Finding:** All tracking approaches achieve 100% decision consistency. proj's value is efficiency (6-10x faster context recovery), not accuracy improvement.

- [Full Case Study (Markdown)](03-multi-model-comparison-jan30.md)

| Metric | proj | Comments | Nothing |
|--------|------|----------|---------|
| Context recovery time | <5s | 30-60s | 45-90s |
| Decision consistency | 100% | 100% | 100% |
| Cost overhead | +20-25% | baseline | lowest |
| Code quality | Equal | Equal | Equal |

**Key insight:** AIs adapt creatively without instructions (Claude read output files, Codex added code comments spontaneously), but proj provides the fastest and most reliable context recovery.

| Model | Total Cost (all studies) | Notes |
|-------|-------------------------|-------|
| Claude | $1.68 | Efficient, concise |
| Codex | $18.41 | Verbose, thorough |

### 04. Long-term Context Tracking (January 29-30, 2026)

**Test:** 12-session deep dive comparing proj vs. code comments vs. no instructions (Claude only)

**Finding:** proj's efficiency advantages compound over time. After session 6-7, proj becomes both faster AND cheaper than alternatives.

- [Full Case Study (Markdown)](04-long-term-context-jan30.md)

| Metric | proj | Comments | Nothing |
|--------|------|----------|---------|
| Total time (12 sessions) | 44.5 min | 51.5 min | 41.6 min |
| Total cost | **$0.63** | $0.65 | $0.64 |
| Decisions tracked | 17 | 22 | 0 explicit |
| Contradictions | 0 | 0 | 7 (self-corrected) |

**The Crossover Effect:**

| Phase | Sessions | Fastest Approach |
|-------|----------|------------------|
| Foundation | 1-3 | nothing (no overhead) |
| Growth | 4-6 | nothing (minimal context) |
| Complexity | 7-9 | **proj** (context recovery wins) |
| Maturity | 10-12 | **proj** (advantage maintained) |

**Key insight:** For projects spanning 7+ sessions, proj pays for itself in both time and cost. The tracking overhead is constant; the context recovery savings compound.

## Future Studies

## Summary

| Study | Comparison | Key Finding |
|-------|------------|-------------|
| 01 | proj vs. nothing | Context recovery impossible without tracking |
| 02 | proj vs. code comments | Equal accuracy, proj more efficient |
| 03 | Multi-model comparison | proj efficiency is 6-10x faster, not accuracy |
| 04 | Long-term (12 sessions) | **proj becomes cheapest after session 6-7** |

The combined findings suggest:
1. **Some form of tracking is essential** for multi-session AI work
2. **proj provides efficiency gains** over manual documentation methods
3. **Accuracy depends on documentation discipline**, not the specific tool
4. **proj's advantages compound** - for longer projects, it saves both time and money
