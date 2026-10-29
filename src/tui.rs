//! Human viewer and annotation interface for an agent-owned `TODO.md`.

use crossterm::{
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lomo::{load, matches_filter, Document, Status, Task};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::collections::HashSet;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            DisableBracketedPaste,
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );
    }
}

enum Mode {
    Normal,
    Filter,
    Note,
    Help,
}

struct Row {
    task_id: Option<String>,
    status: Option<Status>,
    annotation: Option<String>,
    depth: usize,
    label: String,
    kind: RowKind,
}

enum RowKind {
    Header,
    Task,
}

struct App {
    path: PathBuf,
    doc: Document,
    selected: usize,
    collapsed: HashSet<String>,
    mode: Mode,
    filter: String,
    draft: String,
    message: String,
    inspector_scroll: u16,
    dirty_reload: bool,
    last_disk: String,
}

pub fn run(path: &Path) -> io::Result<()> {
    let doc = load(path).map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                err.kind(),
                format!(
                    "{} does not exist; ask the agent to create it",
                    path.display()
                ),
            )
        } else {
            err
        }
    })?;
    let last_disk = doc.serialize();
    let mut app = App {
        path: path.to_path_buf(),
        doc,
        selected: 0,
        collapsed: HashSet::new(),
        mode: Mode::Normal,
        filter: String::new(),
        draft: String::new(),
        message: format!("{}", path.display()),
        inspector_scroll: 0,
        dirty_reload: false,
        last_disk,
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |result| {
            let _ = tx.send(result);
        },
        notify::Config::default(),
    )
    .map_err(|err| io::Error::other(err.to_string()))?;
    let watch_dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(|err| io::Error::other(err.to_string()))?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste,
        crossterm::cursor::Hide
    )?;
    let _guard = TermGuard;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        while let Ok(event) = rx.try_recv() {
            if let Ok(event) = event {
                if event_is_ours(&event, path) {
                    app.dirty_reload = true;
                }
            }
        }
        if app.dirty_reload {
            app.dirty_reload = false;
            reload_if_external(&mut app)?;
        }

        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(120))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(&mut app, key)? {
                        break;
                    }
                }
                Event::Paste(text) if matches!(app.mode, Mode::Note) => app.draft.push_str(&text),
                _ => {}
            }
        }
    }
    Ok(())
}

fn event_is_ours(event: &notify::Event, path: &Path) -> bool {
    let name = path.file_name();
    event.paths.iter().any(|p| p.file_name() == name) || event.paths.iter().any(|p| p == path)
}

fn reload_if_external(app: &mut App) -> io::Result<()> {
    if !matches!(app.mode, Mode::Normal | Mode::Help) {
        app.dirty_reload = true;
        return Ok(());
    }
    let fresh = match load(&app.path) {
        Ok(doc) => doc,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            app.message = "waiting for the agent to restore the file".into();
            return Ok(());
        }
        Err(err) => return Err(err),
    };
    let text = fresh.serialize();
    if text == app.last_disk {
        return Ok(());
    }
    let selected_id = selected_task_id(app);
    app.doc = fresh;
    app.last_disk = text;
    if let Some(id) = selected_id {
        if let Some(pos) = visible_rows(app)
            .iter()
            .position(|row| row.task_id.as_deref() == Some(id.as_str()))
        {
            app.selected = pos;
        }
    }
    app.message = format!("reloaded {}", app.path.display());
    Ok(())
}

fn write_note(app: &mut App, id: &str, note: &str) -> Result<(), lomo::LomoError> {
    let selected_id = selected_task_id(app);
    let doc = lomo::annotate(&app.path, id, note)?;
    app.last_disk = doc.serialize();
    app.doc = doc;
    if let Some(id) = selected_id {
        if let Some(position) = visible_rows(app)
            .iter()
            .position(|row| row.task_id.as_deref() == Some(id.as_str()))
        {
            app.selected = position;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true);
    }
    match app.mode {
        Mode::Help => {
            app.mode = Mode::Normal;
            return Ok(false);
        }
        Mode::Filter => return filter_key(app, key),
        Mode::Note => return note_key(app, key),
        Mode::Normal => {}
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
        KeyCode::Char('?') => app.mode = Mode::Help,
        KeyCode::Char('j') | KeyCode::Down => move_sel(app, 1),
        KeyCode::Char('k') | KeyCode::Up => move_sel(app, -1),
        KeyCode::Char('h') | KeyCode::Left => collapse(app, true),
        KeyCode::Char('l') | KeyCode::Right => collapse(app, false),
        KeyCode::Char('g') => app.selected = 0,
        KeyCode::Char('G') => {
            let n = visible_rows(app).len();
            app.selected = n.saturating_sub(1);
        }
        KeyCode::Tab => {
            app.inspector_scroll = app.inspector_scroll.saturating_add(1);
        }
        KeyCode::BackTab => {
            app.inspector_scroll = app.inspector_scroll.saturating_sub(1);
        }
        KeyCode::Char('n') => {
            app.draft = selected_task_id(app)
                .and_then(|id| app.doc.task(&id))
                .and_then(|task| task.annotation.clone())
                .unwrap_or_default();
            app.mode = Mode::Note;
        }
        KeyCode::Char('/') => {
            app.draft = app.filter.clone();
            app.mode = Mode::Filter;
        }
        KeyCode::Char('r') => {
            app.dirty_reload = true;
            app.last_disk.clear();
            app.message = "reloading".into();
        }
        KeyCode::Char('J') => {
            app.inspector_scroll = app.inspector_scroll.saturating_add(1);
        }
        KeyCode::Char('K') => {
            app.inspector_scroll = app.inspector_scroll.saturating_sub(1);
        }
        _ => {}
    }
    Ok(false)
}

fn move_sel(app: &mut App, delta: i32) {
    let n = visible_rows(app).len();
    if n == 0 {
        app.selected = 0;
        return;
    }
    let next = app.selected as i32 + delta;
    app.selected = next.clamp(0, n as i32 - 1) as usize;
    app.inspector_scroll = 0;
}

fn collapse(app: &mut App, collapse: bool) {
    let Some(id) = selected_task_id(app) else {
        return;
    };
    if collapse {
        if !app.doc.children_ids(&id).is_empty() {
            app.collapsed.insert(id.clone());
        }
    } else {
        app.collapsed.remove(&id);
    }
}

fn note_key(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.message = "note canceled".into();
        }
        KeyCode::Enter
            if key
                .modifiers
                .intersects(KeyModifiers::ALT | KeyModifiers::SHIFT) =>
        {
            app.draft.push('\n')
        }
        KeyCode::Enter => {
            let note = app.draft.clone();
            let Some(id) = selected_task_id(app) else {
                app.message = "select a task with an agent-assigned ID to annotate".into();
                return Ok(false);
            };
            if let Err(err) = write_note(app, &id, &note) {
                app.message = err.to_string();
            } else {
                app.mode = Mode::Normal;
                app.draft.clear();
                app.message = format!("noted on {id}");
            }
        }
        KeyCode::Backspace => {
            app.draft.pop();
        }
        KeyCode::Char(c) => app.draft.push(c),
        _ => {}
    }
    Ok(false)
}

fn filter_key(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.draft.clear();
            app.filter.clear();
            app.selected = 0;
        }
        KeyCode::Enter => {
            app.filter = app.draft.trim().to_string();
            app.mode = Mode::Normal;
            app.selected = 0;
            app.message = if app.filter.is_empty() {
                "filter cleared".into()
            } else {
                format!("filter: {}", app.filter)
            };
        }
        KeyCode::Backspace => {
            app.draft.pop();
        }
        KeyCode::Char(c) => app.draft.push(c),
        _ => {}
    }
    Ok(false)
}

fn selected_task_id(app: &App) -> Option<String> {
    visible_rows(app)
        .get(app.selected)
        .and_then(|row| row.task_id.clone())
}

fn visible_rows(app: &App) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut hidden_below: Option<usize> = None;
    for line in &app.doc.lines {
        match line {
            lomo::LineKind::Raw(raw) => {
                if hidden_below.is_some() {
                    continue;
                }
                if is_display_header(raw) {
                    rows.push(Row {
                        task_id: None,
                        status: None,
                        annotation: None,
                        depth: 0,
                        label: raw.trim().to_string(),
                        kind: RowKind::Header,
                    });
                }
            }
            lomo::LineKind::Task(task) => {
                if let Some(width) = hidden_below {
                    if task.indent_width > width {
                        continue;
                    }
                    hidden_below = None;
                }
                if task
                    .id
                    .as_ref()
                    .is_some_and(|id| app.collapsed.contains(id))
                {
                    hidden_below = Some(task.indent_width);
                }
                if !app.filter.is_empty() && !task_visible(app, task) {
                    continue;
                }
                let marker = if task
                    .id
                    .as_ref()
                    .is_some_and(|id| app.collapsed.contains(id))
                    && !app
                        .doc
                        .children_ids(task.id.as_deref().unwrap_or(""))
                        .is_empty()
                {
                    "▸ "
                } else if !app
                    .doc
                    .children_ids(task.id.as_deref().unwrap_or(""))
                    .is_empty()
                {
                    "▾ "
                } else {
                    "  "
                };
                let pri = task.priority.map(|p| format!(" {}", p)).unwrap_or_default();
                rows.push(Row {
                    task_id: task.id.clone(),
                    status: Some(task.status),
                    annotation: task.annotation.clone(),
                    depth: task.indent_width / 2,
                    label: format!("{marker}{pri} {}", task.description),
                    kind: RowKind::Task,
                });
            }
        }
    }
    rows
}

fn task_visible(app: &App, task: &Task) -> bool {
    if matches_filter(&app.doc, task, &app.filter) {
        return true;
    }
    // Keep ancestors of a match so the tree still reads.
    let mut cursor = task.id.clone();
    let mut guard = 0;
    while let Some(id) = cursor {
        if matches_filter(&app.doc, app.doc.task(&id).unwrap(), &app.filter) {
            return true;
        }
        // descendants
        if app
            .doc
            .tasks()
            .skip_while(|t| t.id.as_deref() != Some(id.as_str()))
            .skip(1)
            .take_while(|t| t.indent_width > app.doc.task(&id).unwrap().indent_width)
            .any(|t| matches_filter(&app.doc, t, &app.filter))
        {
            return true;
        }
        cursor = app.doc.parent_id(&id);
        guard += 1;
        if guard > 64 {
            break;
        }
    }
    false
}

fn is_display_header(raw: &str) -> bool {
    let trimmed = raw.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&hashes)
}

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from("  ┌───────────────┐"),
            Line::from("  │ l │ o │ m │ o │  Low Overhead Markdown Overwatch"),
            Line::from(
                "  └───────────────┘  Copyright (c) 2026 John Anthony Mariquit <john@mariquit.com>",
            ),
            Line::from(""),
        ])
        .style(Style::default().fg(Color::Gray)),
        chunks[0],
    );
    draw_list(frame, body[0], app);
    draw_inspector(frame, body[1], app);
    draw_status(frame, chunks[2], app);

    match app.mode {
        Mode::Filter => draw_prompt(frame, "filter", &app.draft),
        Mode::Note => draw_prompt(
            frame,
            "note: Enter saves, Alt+Enter adds a line",
            &app.draft,
        ),
        Mode::Help => draw_help(frame),
        Mode::Normal => {}
    }
}

fn draw_list(frame: &mut Frame, area: Rect, app: &App) {
    let rows = visible_rows(app);
    let title = format!(" tasks — {} ", app.path.display());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if rows.is_empty() {
        let empty = Paragraph::new("No matching tasks. The agent maintains TODO.md.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }
    let selected = app.selected.min(rows.len().saturating_sub(1));
    let height = inner.height as usize;
    let start = selected
        .saturating_sub(height.saturating_sub(1) / 2)
        .min(rows.len().saturating_sub(height));
    let mut lines = Vec::new();
    for (offset, row) in rows.iter().enumerate().skip(start).take(height) {
        let selected_row = offset == selected;
        lines.push(row_line(row, selected_row));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn row_line(row: &Row, selected: bool) -> Line<'static> {
    let prefix = match row.kind {
        RowKind::Header => String::new(),
        RowKind::Task => format!(
            "{} {} ",
            row.task_id.as_deref().unwrap_or("(no id)"),
            row.status.unwrap()
        ),
    };
    let style = row_style(row, selected);
    let label_style = match row.kind {
        RowKind::Task => style.fg(Color::Gray),
        RowKind::Header => style,
    };
    let mut spans = vec![
        Span::styled(prefix, style),
        Span::styled(
            format!("{}{}", "  ".repeat(row.depth), row.label),
            label_style,
        ),
    ];
    if let Some(annotation) = &row.annotation {
        let mut style = Style::default().fg(Color::Yellow);
        if selected {
            style = style.bg(Color::DarkGray);
        }
        spans.push(Span::styled(
            format!(" {}", annotation.replace('\n', " ⏎ ")),
            style,
        ));
    }
    Line::from(spans)
}

fn row_style(row: &Row, selected: bool) -> Style {
    let base = match row.kind {
        RowKind::Header => Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        RowKind::Task => match row.status {
            Some(Status::Pending) => Style::default().fg(Color::Gray),
            Some(Status::InProgress) => Style::default().fg(Color::Yellow),
            Some(Status::InReview) => Style::default().fg(Color::Cyan),
            Some(Status::Discovered) => Style::default().fg(Color::Magenta),
            Some(Status::Blocked) => Style::default().fg(Color::Blue),
            Some(Status::SafetyGate) => {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            }
            Some(Status::Completed) => Style::default().fg(Color::Green),
            Some(Status::Canceled) => Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::CROSSED_OUT),
            None => Style::default(),
        },
    };
    if selected {
        base.bg(Color::DarkGray).add_modifier(Modifier::BOLD)
    } else {
        base
    }
}

fn draw_inspector(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" inspector ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = inspector_text(app);
    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .scroll((app.inspector_scroll, 0));
    frame.render_widget(paragraph, inner);
}

fn inspector_text(app: &App) -> Vec<Line<'static>> {
    let Some(id) = selected_task_id(app) else {
        return vec![Line::from(Span::styled(
            "Select a task.",
            Style::default().fg(Color::DarkGray),
        ))];
    };
    let Some(task) = app.doc.task(&id) else {
        return Vec::new();
    };
    let mut lines = vec![
        Line::from(Span::styled(
            format!("id {id}    line {}", task.line + 1),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(vec![
            Span::styled("status  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} {}", task.status, task.status.name()),
                status_style(task.status),
            ),
        ]),
        Line::from(vec![
            Span::styled("priority  ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!(
                "{} {}",
                task.effective_priority(),
                task.effective_priority().name()
            )),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            task.description.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];
    if let Some(annotation) = &task.annotation {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "annotation",
            Style::default().fg(Color::DarkGray),
        )));
        for line in annotation.split('\n') {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Yellow),
            )));
        }
    }
    let notes = app.doc.context_lines(&id);
    if !notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "notes",
            Style::default().fg(Color::DarkGray),
        )));
        for (_, note) in notes {
            lines.push(Line::from(note.to_string()));
        }
    }
    if let Some(parent) = app.doc.parent_id(&id) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("parent {parent}"),
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines
}

fn status_style(status: Status) -> Style {
    match status {
        Status::InProgress => Style::default().fg(Color::Yellow),
        Status::InReview => Style::default().fg(Color::Cyan),
        Status::Discovered => Style::default().fg(Color::Magenta),
        Status::Blocked => Style::default().fg(Color::Blue),
        Status::SafetyGate => Style::default().fg(Color::Red),
        Status::Completed => Style::default().fg(Color::Green),
        Status::Canceled => Style::default().fg(Color::DarkGray),
        Status::Pending => Style::default().fg(Color::Gray),
    }
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let keys = " j/k move  h/l fold  n annotate  / filter  ? help  q quit ";
    let line = Line::from(vec![
        Span::styled(keys, Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(app.message.clone(), Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_prompt(frame: &mut Frame, title: &str, value: &str) {
    let height = (value.split('\n').count().min(10) + 2) as u16;
    let area = centered(frame.area(), 60, height);
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(value.to_string()), inner);
}

fn draw_help(frame: &mut Frame) {
    let area = centered(frame.area(), 64, 16);
    frame.render_widget(Clear, area);
    let text = "\
j/k or arrows   move
h/l or arrows   collapse / expand
n               edit annotation (including priority suggestions)
Alt+Enter       new line in a note; Enter saves
/               filter (p:0, status:blocked, text)
Tab / J K       scroll inspector
r               reload
q               quit

Any key closes this help.";
    let block = Block::default()
        .title(" help ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
