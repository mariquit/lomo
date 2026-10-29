# LOMO: Low Overhead Markdown Overwatch

LOMO is a terminal viewer and human annotation interface for an **agent-owned
`TODO.md`**. The agent creates the file, includes rich context, and maintains its
tasks, IDs, statuses, and priorities directly. The human watches that file and
leaves feedback without taking over the agent's TODO list.

```text
  ┌───────────────┐
  │ l │ o │ m │ o │  Low Overhead Markdown Overwatch
  └───────────────┘
```

## What it does

- Displays the task tree, statuses, priorities, and task context.
- Refreshes when the agent changes the file.
- Supports folding, search, status/priority filters, and an inspector.
- Edits human annotations with `n`, preserving the existing task text.
- Provides a guidance skill describing the Markdown conventions.

LOMO does **not** create `TODO.md`, generate IDs, add tasks, change statuses or
priorities, enforce a lifecycle, run tests, make commits, or orchestrate agents.
It has no task-management CLI, hooks, session state, or Git integration.

## Human feedback

```markdown
# Login redesign

Context and implementation plans written by the agent belong here.

- [.] (p2) #a1b2c3: Replace the login flow [annotation]**Please prioritize the migration path first.**[/annotation]
  - Context/reference: Existing sessions must survive the replacement. See [the design](design.md).
  - Done when: New logins use the replacement flow and existing sessions still authenticate.
```

Humans suggest changes through notes. The agent reads/re-reads the file and
updates priority, progress, or status itself. Task creation and editing are
always the agent's responsibility, using its normal file tools—not LOMO.
The optional annotation is at the end of the task description, enclosed by
`[annotation]` and `[/annotation]`. Its contents can use Markdown. In the TUI,
the markers are hidden and the contents are displayed in yellow.

Press `n` to edit the annotation. The input box shows only its contents, not
the markers. `Enter` saves the contents with the markers restored, replacing
the existing annotation rather than adding a second one. Use `Alt+Enter` for a
new line or paste multiline Markdown; the closing marker ends that annotation.

## Task notation

```text
- [STATUS] (pPRIORITY) #hhhhhh: Description [annotation]Markdown text[/annotation]
```

The agent chooses unique six-hex IDs, writes each as `#hhhhhh:`, and keeps each ID with its task. Priority
is required (`p0`–`p4`). There is no default. The entire annotation field
is optional. The marker strings are reserved delimiters, not part of the note's
editable contents. The headline stays one line. Required supporting bullets are
`Context/reference` and `Done when`. Optional bullets are `Dependencies`,
`Constraints`, and `Locations`; omit them when they add nothing. Nest child
tasks with indentation. Supporting bullets are not child tasks.

| Status | Meaning |
| --- | --- |
| `[ ]` | Pending |
| `[.]` | In progress |
| `[:]` | In review |
| `[+]` | Discovered work |
| `[?]` | Blocked |
| `[!]` | Sensitive work needing human confirmation |
| `[x]` | Completed |
| `[~]` | Canceled |

These statuses are displayed, not enforced. Lines without IDs are shown as-is;
the agent needs to assign IDs before humans can annotate those tasks.

## Use

```bash
lomo                       # View the agent's existing TODO.md
lomo --file path/TODO.md    # View another existing file
lomo setup                 # Install the guidance skill for OpenCode
lomo setup claude          # Or: codex, cursor
```

`LOMO_FILE` also selects the file. LOMO never creates a missing board.

| Key | Action |
| --- | --- |
| `j` / `k`, arrows | Move |
| `h` / `l`, arrows | Collapse / expand |
| `n` | Edit annotation contents, including priority suggestions or cancellation requests |
| `Alt+Enter` in a note | New line (`Enter` saves the note) |
| `/` | Search/filter (`p:0`, `status:blocked`, or text) |
| `Tab`, `J` / `K` | Scroll the inspector |
| `r` | Reload |
| `?` | Help |
| `q` / `Esc` | Quit |

There are no task creation, status, or priority-changing keys.

## Guidance and installation

[`assets/SKILL.md`](assets/SKILL.md) is the embedded guidance skill. It tells the
agent to own the file, maintain the notation, read human comments, and consider
an atomic Git commit when completing a task. It does not impose a CLI workflow.

See [GETTING_STARTED.md](GETTING_STARTED.md) before the first todo list. The lomo skill is not loaded automatically.

See [INSTALL.md](INSTALL.md) for installation and upgrading older setups.

## License

Copyright (c) 2026 John Anthony Mariquit <john@mariquit.com>

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

## Contributing

Contributions are welcome! Please read our [CONTRIBUTING.md](./CONTRIBUTING.md) guide for details on our coding standards, non-destructive parsing requirements, and pull request process.
