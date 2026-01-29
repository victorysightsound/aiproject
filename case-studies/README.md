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

## Planned Studies

### 03. No Instructions Test (Planned)

**Question:** What happens when AI agents receive no tracking or documentation instructions at all?

**Metrics to measure:**
- Does AI spontaneously create documentation?
- How much context is lost between sessions?
- What accuracy degradation occurs?
- When does the AI contradict itself?

This will provide the clearest measurement of proj's value proposition.

## Summary

| Study | Comparison | Key Finding |
|-------|------------|-------------|
| 01 | proj vs. nothing | Context recovery impossible without tracking |
| 02 | proj vs. code comments | Equal accuracy, proj more efficient |
| 03 | No instructions | (planned) |

The combined findings suggest:
1. **Some form of tracking is essential** for multi-session AI work
2. **proj provides efficiency gains** over manual documentation methods
3. **Accuracy depends on documentation discipline**, not the specific tool
