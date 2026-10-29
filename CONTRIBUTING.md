# Contributing to LOMO

Thank you for your interest in contributing to **LOMO (Low Overhead Markdown Overwatch)**! 

LOMO lets humans view and annotate an agent-owned `TODO.md`. The agent creates
and maintains the file directly, including tasks, IDs, statuses, and priorities.
LOMO must not become a task manager, orchestrator, lifecycle enforcer, test
runner, or Git manager. Human priority suggestions are annotations, not controls.

---

## 1. Code of Conduct

We aim to foster a welcoming, respectful, and collaborative environment. Please treat all contributors and maintainers with respect and constructive feedback.

---

## 2. Project Architecture Overview

Before making changes, it helps to understand the two main components of LOMO:

1. **Rust TUI Binary (`lomo`)**:
   - **Parser / AST**: Responsible for line-by-line parsing of `TODO.md`. Must preserve non-task lines (headers, code blocks, raw markdown) byte-for-byte.
   - **Watcher (`notify`)**: Handles cross-platform, asynchronous filesystem watching for real-time interface reloads.
   - **UI Layer (`ratatui` + `crossterm`)**: Renders the task tree, status badges, priorities, inspector, and human annotation dialog. It cannot create tasks or change status/priority.
2. **Agent Guidance (`assets/SKILL.md`)**:
   - Explains the Markdown notation and asks agents to read human notes and keep their TODO current using normal file tools. It recommends atomic commits but installs no enforcement hooks or persistent instruction blocks.

---

## 3. Development Setup

### Prerequisites
- **Rust**: Latest stable toolchain (`rustup update stable`).
- **Git**: For version control.

### Building & Running Locally

```bash
# Clone the repository
git clone https://github.com/mariquit/lomo.git
cd lomo

# Build the project
cargo build

# Run LOMO locally against a test TODO.md
cargo run -- --file /path/to/an/existing/TODO.md
```

---

## 4. Key Guidelines & Principles

### A. Non-Destructive File Writes
Viewing/reloading must never write the file, create a missing board, or fill in
IDs. The only write operation is editing the trailing `[annotation]...[/annotation]` field of an existing
task. It should reload the latest file first and preserve existing content,
including headings, context, task fields, comments, and newline styles. Do not
add locks, checkpoints, session state, or mutation APIs for agent task management.

### B. Regex & Syntax Consistency
All parsing changes must conform strictly to the canonical task line regex:
```text
(?s)^(?P<indent>[ \t]*)-\s*\[(?P<status>[ .:\+?!x~])\]\s*\((?P<priority>p[0-4])\)\s*(?P<id>#[0-9a-fA-F]{6}):\s*(?P<description>.*?)(?:\s+\[annotation\](?P<annotation>.*?)\[/annotation\])?\s*$
```
If you propose changing task line syntax or status flags, you must update:
1. `src/lib.rs`
2. `README.md`
3. `assets/SKILL.md`

### C. Performance & Dependencies
LOMO is built to be **Low Overhead**. Avoid adding large dependency trees or blocking operations in the main TUI render loop.

---

## 5. How to Contribute

### Reporting Bugs
- Check existing [GitHub Issues](https://github.com/mariquit/lomo/issues) to avoid duplicates.
- Include your operating system, terminal emulator, Rust version, and a reproducible `TODO.md` snippet.

### Submitting Pull Requests (PRs)
1. **Fork & Branch**: Create a feature branch off `main` (e.g., `feature/custom-themes` or `fix/parser-utf8`).
2. **Keep PRs Focused**: Limit each PR to a single logical bug fix or feature.
3. **Add Tests**: Include unit tests for parser changes, edge cases, or keybindings.
4. **Format & Lint**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```
5. **Open a PR**: Fill out the PR template detailing what was changed and how to test it.

---

## 6. License

By contributing to LOMO, you agree that your contributions will be licensed under the project's [MIT License](./LICENSE).
