//! Read an agent-owned TODO.md and edit its marked annotations without managing tasks.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Pending,
    InProgress,
    InReview,
    Discovered,
    Blocked,
    SafetyGate,
    Completed,
    Canceled,
}

impl Status {
    pub fn symbol(self) -> char {
        match self {
            Self::Pending => ' ',
            Self::InProgress => '.',
            Self::InReview => ':',
            Self::Discovered => '+',
            Self::Blocked => '?',
            Self::SafetyGate => '!',
            Self::Completed => 'x',
            Self::Canceled => '~',
        }
    }

    pub fn from_symbol(symbol: char) -> Option<Self> {
        Some(match symbol {
            ' ' => Self::Pending,
            '.' => Self::InProgress,
            ':' => Self::InReview,
            '+' => Self::Discovered,
            '?' => Self::Blocked,
            '!' => Self::SafetyGate,
            'x' | 'X' => Self::Completed,
            '~' => Self::Canceled,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in-progress",
            Self::InReview => "in-review",
            Self::Discovered => "discovered",
            Self::Blocked => "blocked",
            Self::SafetyGate => "safety-gate",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "pending" | "todo" | " " => Some(Self::Pending),
            "in-progress" | "doing" | "." => Some(Self::InProgress),
            "in-review" | "review" | ":" => Some(Self::InReview),
            "discovered" | "+" => Some(Self::Discovered),
            "blocked" | "?" => Some(Self::Blocked),
            "safety-gate" | "gate" | "!" => Some(Self::SafetyGate),
            "completed" | "done" | "x" => Some(Self::Completed),
            "canceled" | "cancelled" | "~" => Some(Self::Canceled),
            _ => None,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.symbol())
    }
}

/// A displayed priority authored by the agent. LOMO does not change it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl Priority {
    pub fn level(self) -> u8 {
        self as u8
    }
    pub fn token(self) -> &'static str {
        match self {
            Self::P0 => "p0",
            Self::P1 => "p1",
            Self::P2 => "p2",
            Self::P3 => "p3",
            Self::P4 => "p4",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::P0 => "critical",
            Self::P1 => "high",
            Self::P2 => "normal",
            Self::P3 => "low",
            Self::P4 => "lowest",
        }
    }
    pub fn from_token(token: &str) -> Option<Self> {
        match token.trim().to_ascii_lowercase().as_str() {
            "p0" | "0" => Some(Self::P0),
            "p1" | "1" => Some(Self::P1),
            "p2" | "2" => Some(Self::P2),
            "p3" | "3" => Some(Self::P3),
            "p4" | "4" => Some(Self::P4),
            _ => None,
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.token())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// Assigned by the agent; absent IDs are displayed, never filled in by LOMO.
    pub id: Option<String>,
    pub line: usize,
    /// Last physical line, including a multiline annotation if present.
    pub end_line: usize,
    pub indent: String,
    pub indent_width: usize,
    pub status: Status,
    pub priority: Option<Priority>,
    pub description: String,
    pub annotation: Option<String>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineKind {
    Task(Task),
    Raw(String),
}

impl LineKind {
    pub fn raw(&self) -> &str {
        match self {
            Self::Task(task) => &task.raw,
            Self::Raw(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub newline: String,
    pub trailing_newline: bool,
    pub lines: Vec<LineKind>,
}

impl Document {
    pub fn serialize(&self) -> String {
        let mut text = self
            .lines
            .iter()
            .map(LineKind::raw)
            .collect::<Vec<_>>()
            .join(&self.newline);
        if self.trailing_newline && !self.lines.is_empty() {
            text.push_str(&self.newline);
        }
        text
    }

    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.lines.iter().filter_map(|line| match line {
            LineKind::Task(task) => Some(task),
            _ => None,
        })
    }

    pub fn task(&self, id: &str) -> Option<&Task> {
        let id = normalize_id(id);
        self.tasks()
            .find(|task| task.id.as_deref() == Some(id.as_str()))
    }

    pub fn parent_id(&self, id: &str) -> Option<String> {
        let task = self.task(id)?;
        self.tasks()
            .take_while(|other| other.line < task.line)
            .filter(|other| other.indent_width < task.indent_width)
            .last()
            .and_then(|parent| parent.id.clone())
    }

    pub fn children_ids(&self, id: &str) -> Vec<String> {
        let Some(task) = self.task(id) else {
            return Vec::new();
        };
        self.tasks()
            .filter(|other| other.line > task.line)
            .take_while(|other| other.indent_width > task.indent_width)
            .filter_map(|other| other.id.clone())
            .filter(|child| self.parent_id(child).as_deref() == task.id.as_deref())
            .collect()
    }

    /// Indented context after the task/annotation, excluding child tasks.
    pub fn context_lines(&self, id: &str) -> Vec<(usize, &str)> {
        let Some(task) = self.task(id) else {
            return Vec::new();
        };
        let start = task.end_line + 1;
        let mut end = start;
        while let Some(line) = self.lines.get(end) {
            let raw = line.raw();
            if matches!(line, LineKind::Task(_)) || is_fence_marker(raw) {
                break;
            }
            if !raw.trim().is_empty() && indent_width(raw) <= task.indent_width {
                break;
            }
            end += 1;
        }
        while end > start && self.lines[end - 1].raw().trim().is_empty() {
            end -= 1;
        }
        (start..end)
            .map(|index| (index, self.lines[index].raw()))
            .collect()
    }
}

#[derive(Debug)]
pub enum LomoError {
    Io(io::Error),
    Message(String),
}

impl fmt::Display for LomoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}
impl std::error::Error for LomoError {}
impl From<io::Error> for LomoError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Viewing a missing board is an error, not an instruction to create one.
pub fn load(path: &Path) -> io::Result<Document> {
    fs::read_to_string(path).map(|text| parse(&text))
}

const ANNOTATION_START: &str = "[annotation]";
const ANNOTATION_END: &str = "[/annotation]";

fn annotation_span(text: &str) -> Option<std::ops::Range<usize>> {
    let start = text.find(ANNOTATION_START)?;
    let content_start = start + ANNOTATION_START.len();
    let end = content_start + text[content_start..].find(ANNOTATION_END)? + ANNOTATION_END.len();
    if !text[end..].trim().is_empty() {
        return None;
    }
    Some(start..end)
}

/// Edit only the trailing [annotation]...[/annotation] field. The editor supplies
/// its contents, not the markers. All other bytes in the file stay untouched.
pub fn annotate(path: &Path, id: &str, note: &str) -> Result<Document, LomoError> {
    if note.contains(ANNOTATION_START) || note.contains(ANNOTATION_END) {
        return Err(LomoError::Message(
            "enter annotation contents without the reserved annotation markers".into(),
        ));
    }
    let text = fs::read_to_string(path)?;
    let doc = parse(&text);
    let id = normalize_id(id);
    let tasks: Vec<_> = doc
        .tasks()
        .filter(|task| task.id.as_deref() == Some(id.as_str()))
        .collect();
    if tasks.len() != 1 {
        return Err(LomoError::Message(format!(
            "ask the agent to provide a unique task ID for {id}; note not written"
        )));
    }
    let task = tasks[0];
    let start = text
        .split_inclusive('\n')
        .take(task.line)
        .map(str::len)
        .sum::<usize>();
    let len = text[start..]
        .split_inclusive('\n')
        .take(task.end_line - task.line + 1)
        .map(str::len)
        .sum::<usize>();
    let original = &text[start..start + len];
    let range = if let Some(span) = annotation_span(original) {
        span
    } else if original.contains(ANNOTATION_START) || original.contains(ANNOTATION_END) {
        return Err(LomoError::Message(
            "task has malformed annotation markers; annotation not changed".into(),
        ));
    } else {
        let end = task.raw.trim_end().len();
        end..end
    };
    let note = note
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', &doc.newline);
    let separator = if range.is_empty() { " " } else { "" };
    let mut updated = text.clone();
    updated.replace_range(
        start + range.start..start + range.end,
        &format!("{separator}{ANNOTATION_START}{note}{ANNOTATION_END}"),
    );
    fs::write(path, &updated)?;
    Ok(parse(&updated))
}

pub fn parse(input: &str) -> Document {
    let newline = if input.contains("\r\n") { "\r\n" } else { "\n" }.to_string();
    let mut in_fence = false;
    let physical: Vec<_> = input.lines().collect();
    let mut lines = Vec::new();
    let mut index = 0;
    while index < physical.len() {
        let raw = physical[index];
        if is_fence_marker(raw) {
            in_fence = !in_fence;
        } else if !in_fence {
            if let Some(mut task) = parse_task_line(raw) {
                let mut end_line = index;
                if raw.contains(ANNOTATION_START) && annotation_span(raw).is_none() {
                    if let Some(end) = physical[index + 1..]
                        .iter()
                        .position(|line| line.contains(ANNOTATION_END))
                    {
                        let end = index + 1 + end;
                        let block = physical[index..=end].join("\n");
                        if annotation_span(&block).is_some() {
                            task = parse_task_line(&block).unwrap();
                            task.raw = raw.to_string();
                            end_line = end;
                        }
                    }
                }
                task.line = index;
                task.end_line = end_line;
                lines.push(LineKind::Task(task));
                for line in &physical[index + 1..end_line + 1] {
                    lines.push(LineKind::Raw((*line).to_string()));
                }
                index = end_line + 1;
                continue;
            }
        }
        lines.push(LineKind::Raw(raw.to_string()));
        index += 1;
    }
    Document {
        newline,
        trailing_newline: input.ends_with('\n'),
        lines,
    }
}

fn is_fence_marker(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

fn indent_width(line: &str) -> usize {
    line.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 2 } else { 1 })
        .sum()
}

pub fn parse_task_line(line: &str) -> Option<Task> {
    let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
    let after_dash = line[indent.len()..]
        .strip_prefix('-')?
        .trim_start_matches([' ', '\t']);
    let bytes = after_dash.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'[' || bytes[2] != b']' {
        return None;
    }
    let status = Status::from_symbol(bytes[1] as char)?;
    let mut rest = after_dash[3..].trim_start();
    let priority = if rest.starts_with('(') && rest.as_bytes().get(3) == Some(&b')') {
        let priority = rest.get(1..3).and_then(Priority::from_token);
        if priority.is_some() {
            rest = rest[4..].trim_start();
        }
        priority
    } else {
        None
    };
    let id = if rest.starts_with('#')
        && rest
            .as_bytes()
            .get(1..7)
            .is_some_and(|hex| hex.iter().all(u8::is_ascii_hexdigit))
        && rest[7..].starts_with(':')
    {
        let id = normalize_id(&rest[..7]);
        rest = rest[8..].trim_start();
        Some(id)
    } else {
        None
    };
    let rest = rest.trim_end();
    let (description, annotation) = if let Some(span) = annotation_span(rest) {
        (
            rest[..span.start].trim_end(),
            Some(
                rest[span.start + ANNOTATION_START.len()..span.end - ANNOTATION_END.len()]
                    .to_string(),
            ),
        )
    } else {
        (rest, None)
    };
    Some(Task {
        id,
        line: 0,
        end_line: line.lines().count().saturating_sub(1),
        indent: indent.into(),
        indent_width: indent_width(line),
        status,
        priority,
        description: description.into(),
        annotation,
        raw: line.into(),
    })
}

fn normalize_id(id: &str) -> String {
    format!(
        "#{}",
        id.trim().trim_start_matches('#').to_ascii_lowercase()
    )
}

pub fn matches_filter(doc: &Document, task: &Task, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if let Some(priority) = query
        .strip_prefix("p:")
        .or_else(|| query.strip_prefix("priority:"))
    {
        return task.priority == Priority::from_token(priority);
    }
    if let Some(status) = query
        .strip_prefix("status:")
        .or_else(|| query.strip_prefix("s:"))
    {
        return Status::from_name(status) == Some(task.status);
    }
    task.description.to_ascii_lowercase().contains(&query)
        || task.id.as_deref().is_some_and(|id| id.contains(&query))
        || task
            .annotation
            .as_deref()
            .is_some_and(|annotation| annotation.to_ascii_lowercase().contains(&query))
        || doc
            .context_lines(task.id.as_deref().unwrap_or(""))
            .iter()
            .any(|(_, note)| note.to_ascii_lowercase().contains(&query))
}
