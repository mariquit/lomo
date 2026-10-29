# LOMO

This file is the scope boundary for working on the LOMO project.

LOMO is a human viewer and annotation interface for an agent-owned `TODO.md`. The agent creates and edits the file with its normal file tools. The human watches it and leaves feedback. LOMO does not manage the list.

## In scope

- Display the existing task tree, statuses, priorities, IDs, headings, and context.
- Refresh when the agent changes the file.
- Fold, search, filter, and inspect.
- Let the human edit one field: the trailing `[annotation]…[/annotation]` note. Hide the markers, show the contents in yellow, and restore the markers on save.
- Install a guidance skill that tells the agent the notation and to read those notes. Priority changes are suggestions in the note, not LOMO controls.
- `lomo setup` installs that skill and the binary, and removes old LOMO hooks and plugins.

## Out of scope

- Creating a missing `TODO.md`, generating IDs, or adding, editing, or removing tasks.
- Changing status or priority, enforcing a lifecycle, verifying work, running tests, making commits, or orchestrating agents.
- A task-management CLI, hooks, plugins, session state, Git integration, or a doctor command.
- Any annotation format other than `[annotation]…[/annotation]`.
