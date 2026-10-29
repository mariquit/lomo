---
name: lomo
description: Guidance for agents creating and maintaining a TODO.md displayed by LOMO. Covers task IDs, status and priority notation, rich context, and human annotations. Agents read and edit the file directly.
license: MIT
metadata:
  copyright: Copyright (c) 2026 John Anthony Mariquit <john@mariquit.com>
---

# Agent-owned TODO.md

You own the TODO. Create, read, and edit `TODO.md` directly using your normal
file tools. Create a list only when the work needs one. LOMO is a human viewer
and annotation interface, not your task manager or an orchestrator. Do not use
LOMO CLI commands to create or manage tasks.

## Task notation

A task headline is one line. Supporting detail belongs in concise bullets under
that line, not inside the headline.

```text
- [STATUS] (pPRIORITY) #hhhhhh: Headline [annotation]Markdown text[/annotation]
  - Context/reference: required
  - Done when: required
```

Priority is required: `(p0)` through `(p4)`. There is no default. The annotation
and the optional bullets may be omitted. Context and Done when are required.

```markdown
# Authentication

- [.] (p1) #a1b2c3: Replace the login flow [annotation]**Please prioritize the migration path first.**[/annotation]
  - Context/reference: Existing sessions must survive the replacement. See [the design](design.md).
  - Constraints: Do not invalidate sessions created by the current service.
  - Locations: `src/auth/`
  - Done when: New logins use the replacement flow and existing sessions still authenticate.
  - [+] (p1) #d4e5f6: Handle sessions created by the old service
    - Context/reference: Discovered while tracing the current issuer.
    - Done when: Those sessions remain valid or have an explicit migration path.
- [ ] (p2) #789abc: Document deployment steps
  - Context/reference: Operators need the replacement rollout order.
  - Done when: The deployment doc lists the order and the rollback check.
```

- Each task starts with `- [STATUS]`, then a required `(p0)`–`(p4)`,
  then `#` and six hex characters, a colon, and a one-line headline.
- Choose the IDs yourself, keep them unique within the file, and retain an ID
  when its task changes. LOMO does not generate or repair IDs.
- Do not omit priority. Lower numbers indicate greater urgency.
- Nest child tasks with indentation, conventionally two spaces per level.
  Child tasks use the same checkbox notation. Supporting bullets do not.
- Required supporting bullets: `Context/reference` and `Done when`.
- Optional supporting bullets, only when they change the work: `Dependencies`,
  `Constraints`, and `Locations`. Omit an empty optional field. Do not write
  `None` or `N/A`.
- Usually one bullet per field. Wrap long lines naturally. Do not force a field
  onto one physical line, and do not expand it into a specification.
- Record only information that changes how the task is understood or carried out.
  Refer to a design or source instead of copying it. No activity diary, repeated
  summary, or filler. Do not invent details to fill a required field; say what
  is still unresolved.
- `Done when` states observable outcomes, not boilerplate such as "implemented
  correctly."
- Headings and fenced code are context, not tasks.

| Status | Meaning |
| --- | --- |
| `[ ]` | Pending |
| `[.]` | In progress |
| `[:]` | In review or awaiting verification |
| `[+]` | Discovered work |
| `[?]` | Blocked; explain what is needed |
| `[!]` | Sensitive or risky work needing human confirmation |
| `[x]` | Completed |
| `[~]` | Canceled or superseded |

These are conventions for your updates, not a lifecycle enforced by LOMO.
Keep status and progress current as you work. Prefer a single active task for
a single-agent project. Record discovered work where it fits the plan; you
decide how to organize and carry out that work.

## Human annotations

Read or re-read `TODO.md` before working and when changing focus. The human
may annotate it while you work. The human annotation is the trailing
`[annotation]...[/annotation]` field at the end of the task description:

```markdown
- [ ] (p3) #123abc: Improve error messages [annotation]**Please raise this to p1.** See [the revised requirements](requirements.md).[/annotation]
```

Humans do not create tasks or change status/priority through LOMO. They write
annotations, including priority suggestions. Read those notes and update the
plan, priority, or status yourself. Preserve the annotation markers and the
human's enclosed Markdown when editing other task fields, and ask for
clarification when a suggestion is ambiguous. For sensitive `[!]` work, seek
explicit human confirmation before taking the risky action or marking it done.

LOMO displays the enclosed text in yellow without showing the markers. Its
annotation editor shows only the contents and restores the markers on save.
The opening marker follows the description on the task line; multiline contents
can continue until `[/annotation]`. Marker strings are reserved delimiters, not
part of the editable contents. Do not invent an HTML or indented-note format
for human annotations.

You handle implementation and verification using your usual tools. Once a task
is complete, an atomic Git commit containing that task's changes and its TODO
update is recommended. Respect unrelated human work. LOMO does not run tests,
make commits, monitor your diff, or enforce these recommendations.
