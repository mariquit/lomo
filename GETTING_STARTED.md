# Getting started

Copyright (c) 2026 John Anthony Mariquit <john@mariquit.com>

LOMO views an agent-owned `TODO.md`. It does not create that file. Setup installs a short global notation block, so the agent can follow the format without loading the skill. The skill has the full writing rules and is not loaded automatically.

## 1. Install

Install the binary and the guidance skill. See [INSTALL.md](INSTALL.md).

```bash
lomo setup
```

Use `lomo setup claude`, `lomo setup codex`, or `lomo setup cursor` for another agent.

## 2. Load the skill

Ask your agent to load the skill before it creates a list. The global block is only the short notation. For example:

```agent
load the lomo skill
```

```agent
load the lomo skill before you create a todo list
```

## 3. Create the list

Once the skill is loaded, ask your agent to create a todo list. The agent writes `TODO.md` with its normal file tools. LOMO does not create the file, assign IDs, or manage tasks.

## 4. View and annotate

```bash
lomo
```

Press `n` to leave a note on the selected task. The agent reads the file and updates priority, progress, or status itself.
