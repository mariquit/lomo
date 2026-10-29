When a project needs a task list, create and edit `TODO.md` directly with your normal file tools. Do not create one unless the work needs it. Do not use LOMO commands to manage tasks. Use this notation even if LOMO is not running.

A task headline is one line. Put supporting detail in concise bullets under it, not in the headline.

```markdown
- [STATUS] (pPRIORITY) #hhhhhh: Headline [annotation]human note[/annotation]
  - Context/reference: why this exists, or the relevant request, issue, or source.
  - Done when: observable completion conditions.
```

Optional, only when they change the work: Dependencies, Constraints, Locations. Omit empty optional fields. Usually one bullet per field. No filler, activity logs, or copied designs. Do not invent details to fill a field.

Statuses: `[ ]` pending, `[.]` in progress, `[:]` in review, `[+]` discovered, `[?]` blocked, `[!]` needs human confirmation, `[x]` done, `[~]` canceled. Priority `(p0)`–`(p4)` is required. Do not omit it. Choose unique six-hex IDs, write each as `#hhhhhh:`, and keep it with the task. Nested tasks use the same checkbox notation. Supporting bullets are not tasks.

Preserve `[annotation]...[/annotation]` and its contents. Those are human notes. Read them and update the task yourself. For `[!]` work, get explicit confirmation before the risky action or marking it done.
