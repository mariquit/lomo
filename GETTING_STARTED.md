# Getting started

LOMO views an agent-owned `TODO.md`. It does not create that file. After setup, the agent already has the notation. You do not need to load the skill before asking for a todo list.

## 1. Install

Install the binary, then install guidance for your agent. See [INSTALL.md](INSTALL.md).

```bash
lomo setup
```

Use `lomo setup claude`, `lomo setup codex`, or `lomo setup cursor` for another agent.

Setup installs two things:

- A short global notation block in that agent's own instructions. The agent follows it on later turns without loading a skill, and without LOMO running.
- The `lomo` skill, which has the full writing rules. It is not loaded automatically. Load it only when you want those details.

The block is added between `<!-- lomo:start -->` and `<!-- lomo:end -->`. Setup does not replace the rest of an existing instructions file.

| Agent | Global notation |
| --- | --- |
| OpenCode | `~/.config/opencode/AGENTS.md` |
| Claude Code | `~/.claude/CLAUDE.md` |
| Codex | `~/.codex/AGENTS.md` |
| Cursor | `~/.cursor/rules/lomo.mdc` |

## 2. Create the list

Ask your agent to create a todo list. The agent writes `TODO.md` with its normal file tools. LOMO does not create the file, assign IDs, or manage tasks.

The global notation already requires this shape:

```text
- [STATUS] (pPRIORITY) #hhhhhh: Headline
  - Context/reference: why this exists.
  - Done when: observable completion conditions.
```

Priority is required. There is no default. The ID is six hex characters followed by a colon. Supporting bullets are not child tasks.

## 3. View and annotate

```bash
lomo
```

If `TODO.md` is missing, LOMO tells you to run setup, then ask the agent to create the list.

Press `n` to leave a note on the selected task. The agent reads the file and updates priority, progress, or status itself.

## Optional: load the skill

Load the skill when you want the full writing rules, not as a step before the first list. For example:

```text
load the lomo skill
```
