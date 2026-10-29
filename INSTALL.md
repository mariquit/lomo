# Install LOMO

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

`cargo install` itself always uses Cargo's bin (`~/.cargo/bin`, or `$CARGO_HOME/bin`).

## Install guidance for your agent

```bash
lomo setup          # OpenCode, the default
lomo setup claude
lomo setup codex
lomo setup cursor
```

Setup installs the guidance skill, a compact global notation block, and the
binary. It does not add plugins or hooks. The notation block is inserted between
`<!-- lomo:start -->` and `<!-- lomo:end -->`. Existing text outside that block
is preserved. Setup does not write this repository's `AGENTS.md`.

| Agent | Guidance skill | Always-available notation |
| --- | --- | --- |
| OpenCode | `~/.config/opencode/skills/lomo/SKILL.md` (respects `XDG_CONFIG_HOME`) | `~/.config/opencode/AGENTS.md` |
| Claude Code | `~/.claude/skills/lomo/SKILL.md` | `~/.claude/CLAUDE.md` |
| Codex | `~/.agents/skills/lomo/SKILL.md` | `~/.codex/AGENTS.md` |
| Cursor | `~/.cursor/skills/lomo/SKILL.md` | `~/.cursor/rules/lomo.mdc` |

OpenCode steering stays in OpenCode's global config, separate from Claude.
The Codex path is Codex's own global instructions, not this repository's scope file.

The skill has the full writing rules. The global block is the short notation,
so the agent can follow it without loading the skill. Invoke the `lomo` skill
when creating or revising a list.

Setup and `install.sh` place the binary in the first of these directories that
is on `PATH`:

1. `~/.local/bin`
2. `~/.cargo/bin` (`$CARGO_HOME/bin` when that is set)
3. `~/.bun/bin`

If none of them is on `PATH`, the binary goes in `~/.local/bin` and setup prints
a reminder to add it. `LOMO_BIN_DIR` overrides the search. On Windows the file
is `lomo.exe`.

## Upgrading an older setup

Rebuild/reinstall, then rerun setup for **each agent you previously configured**.
Setup removes recognized LOMO-owned hooks and the old OpenCode LOMO plugin, then
refreshes the marked notation block. Other hooks, user instructions, and
configuration are preserved. Invalid JSON or an incomplete marked block is
reported rather than overwritten.

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
