# Install LOMO

Copyright (c) 2026 John Anthony Mariquit <john@mariquit.com>

LOMO views and annotates an existing, agent-owned `TODO.md`. It does not create
the file or manage tasks, priorities, tests, or Git.

## Install the binary

```bash
curl -fsSL https://raw.githubusercontent.com/mariquit/lomo/main/install.sh | sh
```

The installer downloads a tagged release and verifies its `.sha256` asset.
Release installation requires a published `v*` release. Alternatively, download
the binary for your platform from GitHub Releases, or build from source:

```bash
cargo install --path .
```

## Install guidance for your agent

```bash
lomo setup          # OpenCode, the default
lomo setup claude
lomo setup codex
lomo setup cursor
```

Setup writes only the guidance skill and installs/copies the binary. It does
not add plugins, hooks, persistent instruction blocks, or Cursor rules.

| Agent | Guidance skill |
| --- | --- |
| OpenCode | `~/.config/opencode/skills/lomo/SKILL.md` (respects `XDG_CONFIG_HOME`) |
| Claude Code | `~/.claude/skills/lomo/SKILL.md` |
| Codex | `~/.agents/skills/lomo/SKILL.md` |
| Cursor | `~/.cursor/skills/lomo/SKILL.md` |

The skill describes status, priority, IDs, rich context, and human comments.
Agents read and edit `TODO.md` directly. Skill loading depends on the agent;
invoke the `lomo` skill explicitly if it is not picked up automatically.

Setup puts the binary in `~/.bun/bin` when that directory is on `PATH`, otherwise
`~/.local/bin`. On Windows it installs `lomo.exe`. Ensure the destination is on
`PATH`; setup prints a reminder if it is not.

## Upgrading an older setup

Rebuild/reinstall, then rerun setup for **each agent you previously configured**.
Setup removes recognized LOMO-owned hooks, the old OpenCode LOMO plugin, or an
owned Cursor rule. It does not read or write `AGENTS.md` or `CLAUDE.md`. Other
hooks, user instructions, and configuration are preserved. Invalid JSON or an
incomplete marked block is reported rather than overwritten.

Restart the agent afterward so old hooks/plugins/instructions are unloaded.
Restart an older running LOMO TUI too: task creation and priority-changing keys
have been removed. The human now uses annotations for all suggestions.

Old `.git/lomo/` bookkeeping is no longer read or written. Setup does not delete
repository data or rewrite existing `TODO.md` files, including old annotations.

## Overrides

| Variable | Effect |
| --- | --- |
| `LOMO_FILE` | Existing file to display instead of `TODO.md` |
| `LOMO_BIN_DIR` | Binary installation directory |
| `LOMO_OPENCODE_CONFIG` | OpenCode configuration directory |
| `LOMO_CLAUDE_CONFIG` | Claude Code configuration directory |
| `LOMO_CODEX_HOME` / `CODEX_HOME` | Codex configuration directory used for legacy cleanup |
| `LOMO_CURSOR_CONFIG` | Cursor configuration directory |
| `LOMO_AGENTS_SKILLS` | Codex skill directory instead of `~/.agents/skills` |
| `XDG_CONFIG_HOME` | Base for OpenCode configuration |
