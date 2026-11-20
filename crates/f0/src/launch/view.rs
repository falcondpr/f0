//! Draws the launcher's `State`: the f0 wordmark, a search box, the current list as a
//! table, and one quiet line of key hints at the bottom. Only the selected row uses the
//! accent color.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Wrap};

use super::state::{PromptKind, Screen, State};

/// The wordmark, drawn when the window is tall enough for it.
const LOGO: [&str; 5] = [
    "██████  ██████",
    "██      ██  ██",
    "█████   ██  ██",
    "██      ██  ██",
    "██      ██████",
];
/// Below this height the wordmark shrinks to one line, leaving room for the rows.
const LOGO_MIN_HEIGHT: u16 = 26;

/// Wider tables spread a row too far to read it in one glance.
const MAX_TABLE_WIDTH: u16 = 120;
/// Space between two columns.
const COLUMN_GAP: u16 = 3;
/// Lines a table takes besides its rows: borders, the header and the line under it.
const TABLE_CHROME: u16 = 4;
const SEARCH_HEIGHT: u16 = 3;

/// Braille frames of the spinner.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const RUNNING: &str = "●";
const STOPPED: &str = "○";
/// Nerd Font glyphs, from the font f0's Ghostty config sets.
const BRANCH: &str = "\u{e0a0}";
const GIT: &str = "\u{e702}";
const SEARCH: &str = "\u{f002}";

/// The f0 palette for the launcher, light or dark like the bundled configs.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Painted under everything, so the launcher keeps its mode even when the terminal
    /// was not told about it.
    pub background: Color,
    /// Only the selected row uses it.
    pub accent: Color,
    pub text: Color,
    /// Labels and metadata.
    pub muted: Color,
    /// Borders, key hints and the least important metadata.
    pub faint: Color,
    /// Behind the selected row.
    pub selection: Color,
    pub running: Color,
    pub error: Color,
}

impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                background: Color::Rgb(0x1f, 0x1f, 0x1f),
                accent: Color::Rgb(0x4d, 0xaa, 0xfc),
                text: Color::Rgb(0xd4, 0xd4, 0xd4),
                muted: Color::Rgb(0xa8, 0xa8, 0xa8),
                faint: Color::Rgb(0x73, 0x73, 0x73),
                selection: Color::Rgb(0x2a, 0x2d, 0x2e),
                running: Color::Rgb(0x8c, 0xc2, 0x65),
                error: Color::Rgb(0xf1, 0x4c, 0x4c),
            }
        } else {
            Self {
                background: Color::Rgb(0xff, 0xff, 0xff),
                accent: Color::Rgb(0x00, 0x5f, 0xb8),
                text: Color::Rgb(0x1f, 0x1f, 0x1f),
                muted: Color::Rgb(0x6e, 0x6e, 0x6e),
                faint: Color::Rgb(0xa8, 0xa8, 0xa8),
                selection: Color::Rgb(0xee, 0xf4, 0xfb),
                running: Color::Rgb(0x3d, 0x7a, 0x1f),
                error: Color::Rgb(0xc7, 0x2e, 0x2e),
            }
        }
    }
}

/// How long ago `then` was, both in seconds since the Unix epoch.
pub fn ago(now: u64, then: u64) -> String {
    let seconds = now.saturating_sub(then);
    match seconds {
        0..60 => "just now".to_owned(),
        60..3_600 => format!("{}m ago", seconds / 60),
        3_600..86_400 => format!("{}h ago", seconds / 3_600),
        86_400..172_800 => "yesterday".to_owned(),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

/// One cell of a row, in one or more styles.
struct Cell {
    spans: Vec<Span<'static>>,
    /// A folder, which is cut at the start to keep its end, the part that tells it apart.
    folder: bool,
}

impl Cell {
    fn new(spans: Vec<Span<'static>>) -> Self {
        Self {
            spans,
            folder: false,
        }
    }

    fn folder(span: Span<'static>) -> Self {
        Self {
            spans: vec![span],
            folder: true,
        }
    }
}

/// Draws `state` on the whole frame. `now` dates the last use of the tasks.
pub fn draw(frame: &mut Frame, state: &State, palette: Palette, now: u64) {
    let area = frame.area();
    frame.render_widget(
        Block::new().style(Style::new().bg(palette.background)),
        area,
    );
    let width = area.width.saturating_sub(4).min(MAX_TABLE_WIDTH);
    let column = Rect {
        x: area.x + (area.width - width) / 2,
        width,
        ..area
    };
    let tall = area.height >= LOGO_MIN_HEIGHT;
    let header_height = if tall { LOGO.len() as u16 + 4 } else { 3 };
    let search_height = if shows_search(state) {
        SEARCH_HEIGHT
    } else {
        0
    };
    let bottom_lines = bottom_text(state, palette).len().min(6) as u16;
    let [header, search, list, bottom, _, hints] = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Length(search_height),
        Constraint::Min(1),
        Constraint::Length(bottom_lines),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(column);

    frame.render_widget(
        Paragraph::new(header_lines(state, palette, tall)).alignment(Alignment::Center),
        header,
    );
    if search_height > 0 {
        draw_search(frame, state, palette, search);
    }
    draw_list(frame, state, palette, now, list);
    frame.render_widget(
        Paragraph::new(bottom_text(state, palette))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        bottom,
    );
    frame.render_widget(
        Paragraph::new(hints_line(state, palette, hints.width)).alignment(Alignment::Center),
        hints,
    );
}

fn header_lines(state: &State, palette: Palette, tall: bool) -> Vec<Line<'static>> {
    let logo = Style::new().fg(palette.text).add_modifier(Modifier::BOLD);
    let mut lines = vec![Line::default()];
    if tall {
        lines.extend(LOGO.iter().map(|row| Line::from(Span::styled(*row, logo))));
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            section(state),
            Style::new().fg(palette.muted),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled("f0", logo),
            Span::styled(
                format!("  ·  {}", section(state)),
                Style::new().fg(palette.muted),
            ),
        ]));
    }
    lines
}

fn section(state: &State) -> String {
    match &state.screen {
        Screen::Projects => "Projects".to_owned(),
        Screen::Tasks { project } => format!("{project}  ·  Tasks"),
        Screen::AddRepo { .. } => "Add a project".to_owned(),
        Screen::Sessions => "Sessions".to_owned(),
    }
}

/// What the rows of the current list are, one and many.
fn noun(screen: &Screen) -> (&'static str, &'static str) {
    match screen {
        Screen::Projects => ("project", "projects"),
        Screen::Tasks { .. } => ("task", "tasks"),
        Screen::AddRepo { .. } => ("repo", "repos"),
        Screen::Sessions => ("session", "sessions"),
    }
}

/// Rows of the current list, filtered or not.
fn total(state: &State) -> usize {
    match state.screen {
        Screen::Projects => state.projects.len(),
        Screen::Tasks { .. } | Screen::Sessions => state.tasks.len(),
        Screen::AddRepo { .. } => state.repos.len(),
    }
}

fn searching(state: &State) -> bool {
    state
        .prompt
        .as_ref()
        .is_some_and(|prompt| prompt.kind == PromptKind::Filter)
}

/// An empty list has nothing to search, unless a search emptied it.
fn shows_search(state: &State) -> bool {
    total(state) > 0 || !state.filter.is_empty() || searching(state)
}

/// The search box above the table: what is typed, the search kept, or what it searches.
fn draw_search(frame: &mut Frame, state: &State, palette: Palette, area: Rect) {
    let typing = searching(state);
    let border = if typing {
        palette.accent
    } else {
        palette.faint
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let icon = Span::styled(format!("{SEARCH}  "), Style::new().fg(palette.muted));
    // The icon, the cursor and the `/` on the right.
    let room = usize::from(inner.width).saturating_sub(6);
    let mut spans = vec![icon];
    if typing {
        let text = state.prompt.as_ref().map_or("", |p| p.text.as_str());
        spans.push(Span::styled(
            truncate_start(text, room),
            Style::new().fg(palette.text),
        ));
        spans.push(Span::styled("▏", Style::new().fg(palette.accent)));
    } else if state.filter.is_empty() {
        let (_, many) = noun(&state.screen);
        spans.push(Span::styled(
            truncate(&format!("Search {many}…"), room),
            Style::new().fg(palette.faint),
        ));
    } else {
        spans.push(Span::styled(
            truncate(&state.filter, room),
            Style::new().fg(palette.text),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), inner);
    if !typing {
        frame.render_widget(
            Paragraph::new(Span::styled("/", Style::new().fg(palette.faint)))
                .alignment(Alignment::Right),
            inner,
        );
    }
}

/// The columns of the current list: their header and how they share the width.
fn columns(screen: &Screen) -> Vec<(&'static str, Constraint)> {
    match screen {
        Screen::Projects => vec![
            ("Project", Constraint::Fill(2)),
            ("Tasks", Constraint::Length(25)),
            ("Repo", Constraint::Fill(3)),
        ],
        Screen::Tasks { .. } => vec![
            ("Task", Constraint::Fill(2)),
            ("Branch", Constraint::Fill(3)),
            ("Status", Constraint::Length(9)),
            ("Last used", Constraint::Length(9)),
        ],
        Screen::AddRepo { .. } => vec![
            ("Repo", Constraint::Fill(1)),
            ("Folder", Constraint::Fill(2)),
        ],
        Screen::Sessions => vec![
            ("Session", Constraint::Fill(1)),
            ("Status", Constraint::Length(9)),
            ("Last used", Constraint::Length(9)),
        ],
    }
}

fn draw_list(frame: &mut Frame, state: &State, palette: Palette, now: u64, area: Rect) {
    let visible = state.visible();
    if visible.is_empty() {
        let lines = empty_lines(state, palette);
        let top = area.height.saturating_sub(lines.len() as u16) / 3;
        let area = Rect {
            y: area.y + top,
            height: area.height - top,
            ..area
        };
        frame.render_widget(
            Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }

    let rows_fit = usize::from(area.height.saturating_sub(TABLE_CHROME)).max(1);
    let shown = visible.len().min(rows_fit) as u16;
    let rect = Rect {
        height: (shown + TABLE_CHROME).min(area.height),
        ..area
    };
    let faint = Style::new().fg(palette.faint);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(faint)
        .title_bottom(
            Line::from(Span::styled(
                format!(" {} ", count(state, visible.len())),
                faint,
            ))
            .right_aligned(),
        );
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let columns = columns(&state.screen);
    let widths: Vec<usize> = Layout::horizontal(columns.iter().map(|(_, width)| *width))
        .spacing(COLUMN_GAP)
        .split(Rect::new(0, 0, inner.width.saturating_sub(2), 1))
        .iter()
        .map(|cell| usize::from(cell.width))
        .collect();
    let width = usize::from(inner.width);
    let header: Vec<Cell> = columns
        .iter()
        .map(|(title, _)| Cell::new(vec![Span::styled(*title, faint)]))
        .collect();
    let mut lines = vec![row_line(&header, &widths, width), Line::default()];
    let first = state.selected.saturating_sub(rows_fit - 1);
    for (position, &index) in visible.iter().enumerate().skip(first).take(rows_fit) {
        let mut cells = cells(state, index, palette, now);
        let selected = position == state.selected;
        if selected && let Some(title) = cells.first_mut() {
            for span in &mut title.spans {
                span.style = span.style.fg(palette.accent).add_modifier(Modifier::BOLD);
            }
        }
        let mut line = row_line(&cells, &widths, width);
        if selected {
            line = line.patch_style(Style::new().bg(palette.selection));
        }
        lines.push(line);
    }
    frame.render_widget(Paragraph::new(lines), inner);
    // The rule under the header joins the borders.
    let rule = Rect {
        y: inner.y + 1,
        height: 1,
        ..rect
    }
    .intersection(rect);
    let inside = usize::from(rect.width.saturating_sub(2));
    frame.render_widget(
        Paragraph::new(Span::styled(format!("├{}┤", "─".repeat(inside)), faint)),
        rule,
    );
}

/// How many rows the table has, and how many the search keeps.
fn count(state: &State, shown: usize) -> String {
    let total = total(state);
    let (one, many) = noun(&state.screen);
    let noun = if total == 1 { one } else { many };
    let mut text = if shown < total {
        format!("{shown} of {total} {noun}")
    } else {
        format!("{total} {noun}")
    };
    if matches!(state.screen, Screen::Tasks { .. }) {
        let running = state.tasks.iter().filter(|t| t.session.is_some()).count();
        if running > 0 {
            text.push_str(&format!(" · {running} running"));
        }
    }
    text
}

/// `cells` laid out in columns `widths` wide, filling `width` so a selected row is
/// highlighted from border to border.
fn row_line(cells: &[Cell], widths: &[usize], width: usize) -> Line<'static> {
    let gap = " ".repeat(usize::from(COLUMN_GAP));
    let mut spans = vec![Span::raw(" ")];
    for (i, (cell, &room)) in cells.iter().zip(widths).enumerate() {
        if i > 0 {
            spans.push(Span::raw(gap.clone()));
        }
        let fitted = match cell.spans.as_slice() {
            _ if room == 0 => Vec::new(),
            [path] if cell.folder => vec![Span::styled(
                truncate_start(&path.content, room),
                path.style,
            )],
            spans => truncate_spans(spans, room),
        };
        let used: usize = fitted.iter().map(|s| s.content.chars().count()).sum();
        spans.extend(fitted);
        spans.push(Span::raw(" ".repeat(room.saturating_sub(used))));
    }
    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    spans.push(Span::raw(" ".repeat(width.saturating_sub(used))));
    Line::from(spans)
}

fn cells(state: &State, index: usize, palette: Palette, now: u64) -> Vec<Cell> {
    let text = |text: String| Span::styled(text, Style::new().fg(palette.text));
    let muted = |text: String| Span::styled(text, Style::new().fg(palette.muted));
    let faint = |text: String| Span::styled(text, Style::new().fg(palette.faint));
    let running = || Span::styled(RUNNING, Style::new().fg(palette.running));
    let stopped = || Span::styled(STOPPED, Style::new().fg(palette.faint));
    let status = |session: &Option<String>| match session {
        Some(_) => Cell::new(vec![running(), muted(" running".to_owned())]),
        None => Cell::new(vec![stopped(), faint(" stopped".to_owned())]),
    };
    let last_used = |activity: Option<u64>| {
        Cell::new(vec![muted(
            activity.map_or_else(String::new, |then| ago(now, then)),
        )])
    };
    match &state.screen {
        Screen::Projects => {
            let project = &state.projects[index];
            let counts = match project.counts {
                None => vec![faint("slots unknown".to_owned())],
                Some((0, 0)) => vec![faint("no tasks".to_owned())],
                Some((on, off)) => {
                    let mut spans = Vec::new();
                    if on > 0 {
                        spans.extend([running(), muted(format!(" {on} running"))]);
                    }
                    if on > 0 && off > 0 {
                        spans.push(faint("  ".to_owned()));
                    }
                    if off > 0 {
                        spans.extend([stopped(), muted(format!(" {off} stopped"))]);
                    }
                    spans
                }
            };
            vec![
                Cell::new(vec![text(project.name.clone())]),
                Cell::new(counts),
                Cell::folder(faint(project.repo.clone())),
            ]
        }
        Screen::Tasks { .. } => {
            let task = &state.tasks[index];
            let branch = if task.branch.is_empty() {
                Vec::new()
            } else {
                vec![muted(format!("{BRANCH} {}", task.branch))]
            };
            vec![
                Cell::new(vec![text(task.task.clone())]),
                Cell::new(branch),
                status(&task.session),
                last_used(task.activity),
            ]
        }
        Screen::Sessions => {
            let task = &state.tasks[index];
            vec![
                Cell::new(vec![text(task.task.clone())]),
                status(&task.session),
                last_used(task.activity),
            ]
        }
        Screen::AddRepo { .. } => {
            let repo = &state.repos[index];
            let name = repo
                .path
                .file_name()
                .map_or_else(|| repo.label.clone(), |n| n.to_string_lossy().into_owned());
            vec![
                Cell::new(vec![text(format!("{GIT} {name}"))]),
                Cell::folder(faint(repo.label.clone())),
            ]
        }
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Keeps the end of `text`, the part of a folder that tells it apart.
fn truncate_start(text: &str, max: usize) -> String {
    let len = text.chars().count();
    if len <= max {
        return text.to_owned();
    }
    let keep = max.saturating_sub(1);
    let mut out = String::from("…");
    out.extend(text.chars().skip(len - keep));
    out
}

/// Cuts `spans` to `max` characters in all, ending with `…` when something was cut.
fn truncate_spans(spans: &[Span<'static>], max: usize) -> Vec<Span<'static>> {
    let total: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    if total <= max {
        return spans.to_vec();
    }
    let mut left = max.saturating_sub(1);
    let mut out = Vec::new();
    for span in spans {
        let len = span.content.chars().count();
        if len <= left {
            out.push(span.clone());
            left -= len;
        } else {
            let cut: String = span.content.chars().take(left).collect();
            out.push(Span::styled(format!("{cut}…"), span.style));
            return out;
        }
    }
    out
}

fn spinner(state: &State) -> &'static str {
    SPINNER[state.tick % SPINNER.len()]
}

/// The calm screen shown instead of an empty list.
fn empty_lines(state: &State, palette: Palette) -> Vec<Line<'static>> {
    let text = |s: String| Line::from(Span::styled(s, Style::new().fg(palette.text)));
    let muted = |s: String| Line::from(Span::styled(s, Style::new().fg(palette.muted)));
    let action = |key: &'static str, label: &'static str| {
        Line::from(vec![
            Span::styled(
                key,
                Style::new().fg(palette.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {label}"), Style::new().fg(palette.muted)),
        ])
    };
    if let Some(busy) = &state.busy {
        return vec![muted(format!("{} {busy}", spinner(state)))];
    }
    if !state.filter.is_empty() {
        return vec![muted(format!("Nothing matches “{}”", state.filter))];
    }
    match &state.screen {
        Screen::Projects => vec![
            text("Welcome to f0".to_owned()),
            muted("Each task gets its own worktree, tmux session and AI agent.".to_owned()),
            Line::default(),
            action("a", "add a project from your repos"),
        ],
        Screen::Tasks { .. } => vec![
            text("No tasks yet".to_owned()),
            Line::default(),
            action("n", "start a new task"),
        ],
        Screen::AddRepo {
            searching: true, ..
        } => vec![muted(format!("{} Looking for repos…", spinner(state)))],
        Screen::AddRepo { folders, .. } => vec![
            text("No new repos found".to_owned()),
            muted(format!("Looked in {}", folders.join(", "))),
            Line::default(),
            action("p", "type the folder of a repo"),
        ],
        Screen::Sessions => vec![text("No f0 sessions running".to_owned())],
    }
}

/// The prompt, a spinner when busy over a list, and the message.
fn bottom_text(state: &State, palette: Palette) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(prompt) = &state.prompt
        && prompt.kind == PromptKind::Confirm
    {
        lines.push(Line::from(vec![
            Span::styled(prompt.text.clone(), Style::new().fg(palette.text)),
            Span::styled("  y/n", Style::new().fg(palette.accent)),
        ]));
    } else if let Some(prompt) = &state.prompt
        && prompt.kind != PromptKind::Filter
    {
        let label = match prompt.kind {
            PromptKind::Ticket => "Ticket for the new task: ",
            PromptKind::Path | PromptKind::Filter | PromptKind::Confirm => "Repo folder: ",
        };
        lines.push(Line::from(vec![
            Span::styled(label, Style::new().fg(palette.muted)),
            Span::styled(prompt.text.clone(), Style::new().fg(palette.text)),
            Span::styled("▏", Style::new().fg(palette.accent)),
        ]));
    }
    if let Some(busy) = &state.busy
        && !state.visible().is_empty()
    {
        lines.push(Line::from(Span::styled(
            format!("{} {busy}", spinner(state)),
            Style::new().fg(palette.muted),
        )));
    }
    if let Some(message) = &state.message {
        let color = if message.error {
            palette.error
        } else {
            palette.muted
        };
        for line in message.text.lines() {
            lines.push(Line::from(Span::styled(
                line.to_owned(),
                Style::new().fg(color),
            )));
        }
    }
    lines
}

/// The keys that do something on the current screen, the most used first: a narrow
/// window drops the last ones.
pub fn hints(state: &State) -> Vec<(&'static str, &'static str)> {
    if let Some(prompt) = &state.prompt {
        return match prompt.kind {
            PromptKind::Filter => vec![("enter", "keep search"), ("esc", "clear")],
            PromptKind::Ticket => vec![("enter", "create"), ("esc", "cancel")],
            PromptKind::Path => vec![("enter", "add"), ("esc", "cancel")],
            PromptKind::Confirm => vec![("y", "yes"), ("any other key", "no")],
        };
    }
    let any = !state.visible().is_empty();
    let mut hints = Vec::new();
    if any {
        hints.push(("↑↓", "move"));
    }
    match state.screen {
        Screen::Projects => {
            if any {
                hints.push(("enter", "tasks"));
            }
            hints.push(("a", "add"));
            if any {
                hints.push(("e", "edit"));
            }
            hints.push(("d", "doctor"));
        }
        Screen::Tasks { .. } => {
            if any {
                hints.push(("enter", "open"));
            }
            hints.push(("n", "new"));
            if any {
                hints.extend([("c", "close"), ("x", "finish")]);
            }
            hints.extend([("e", "edit project"), ("esc", "back")]);
        }
        Screen::AddRepo { .. } => {
            if any {
                hints.push(("enter", "add"));
            }
            hints.extend([("p", "type a path"), ("esc", "back")]);
        }
        Screen::Sessions => {
            if any {
                hints.push(("enter", "switch"));
            }
        }
    }
    hints.extend([("/", "search"), ("q", "quit")]);
    // Seldom used, so a narrow window drops it first.
    if state.screen == Screen::Projects {
        hints.push(("t", state.appearance.hint()));
    }
    hints
}

/// The hints that fit in `width`, whole: a narrow window drops the last ones.
fn hints_line(state: &State, palette: Palette, width: u16) -> Line<'static> {
    let mut spans = Vec::new();
    let mut used = 0;
    for (i, (key, label)) in hints(state).into_iter().enumerate() {
        let gap = if i > 0 { 3 } else { 0 };
        let len = gap + key.chars().count() + 1 + label.chars().count();
        if used + len > usize::from(width) {
            break;
        }
        used += len;
        if gap > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(Span::styled(key, Style::new().fg(palette.muted)));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::new().fg(palette.faint),
        ));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::launch::state::{Key, ProjectItem, RepoItem, TaskItem};

    const NOW: u64 = 1_000_000;

    fn render(state: &State, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw(frame, state, Palette::new(true), NOW))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn project(name: &str, counts: Option<(usize, usize)>) -> ProjectItem {
        ProjectItem {
            name: name.to_owned(),
            repo: format!("~/src/{name}"),
            counts,
        }
    }

    fn tasks_state() -> State {
        let mut state = State::new(vec![project("web", Some((1, 1)))]);
        state.show_tasks(
            "web".to_owned(),
            vec![
                TaskItem {
                    task: "WEB-1".to_owned(),
                    branch: "feature/WEB-1".to_owned(),
                    path: PathBuf::from("/pool/1/web"),
                    session: Some("web/WEB-1".to_owned()),
                    activity: Some(NOW - 2 * 3_600),
                },
                TaskItem {
                    task: "WEB-22".to_owned(),
                    branch: "fix/WEB-22".to_owned(),
                    path: PathBuf::from("/pool/2/web"),
                    session: None,
                    activity: None,
                },
            ],
        );
        state
    }

    #[test]
    fn tall_windows_show_the_big_wordmark() {
        let screen = render(&State::new(vec![project("api", None)]), 100, 30);
        assert!(screen.contains(LOGO[0]), "{screen}");
        assert!(screen.contains("Projects"), "{screen}");
        let short = render(&State::new(vec![project("api", None)]), 100, 16);
        assert!(!short.contains(LOGO[0]), "{short}");
        assert!(short.contains("f0  ·  Projects"), "{short}");
    }

    #[test]
    fn project_rows_show_counts_and_repo() {
        let state = State::new(vec![
            project("api", Some((2, 1))),
            project("web", Some((0, 0))),
            project("cli", None),
        ]);
        let screen = render(&state, 120, 30);
        assert!(screen.contains("Project"), "{screen}");
        assert!(screen.contains("● 2 running  ○ 1 stopped"), "{screen}");
        assert!(screen.contains("no tasks"), "{screen}");
        assert!(screen.contains("slots unknown"), "{screen}");
        assert!(screen.contains("~/src/api"), "{screen}");
        assert!(screen.contains(" 3 projects "), "{screen}");
        assert!(
            screen.contains("enter tasks   a add   e edit   d doctor   / search   q quit"),
            "{screen}"
        );
    }

    #[test]
    fn only_the_selected_row_is_highlighted() {
        let state = State::new(vec![project("api", None), project("web", None)]);
        let palette = Palette::new(false);
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal
            .draw(|frame| draw(frame, &state, palette, NOW))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let rows = |color: Color, fg: bool| -> Vec<u16> {
            let mut rows: Vec<u16> = (0..buffer.area.height)
                .filter(|&y| {
                    (0..buffer.area.width).any(|x| {
                        let cell = &buffer[(x, y)];
                        if fg {
                            cell.fg == color
                        } else {
                            cell.bg == color
                        }
                    })
                })
                .collect();
            rows.dedup();
            rows
        };
        let highlighted = rows(palette.selection, false);
        assert_eq!(highlighted.len(), 1);
        assert_eq!(rows(palette.accent, true), highlighted);
        let line: String = (0..buffer.area.width)
            .map(|x| buffer[(x, highlighted[0])].symbol())
            .collect();
        assert!(line.contains("api"), "{line}");
    }

    #[test]
    fn task_rows_show_branch_status_and_last_use() {
        let screen = render(&tasks_state(), 100, 30);
        assert!(screen.contains("web  ·  Tasks"), "{screen}");
        let header = screen.lines().find(|l| l.contains("Last used")).unwrap();
        for title in ["Task ", "Branch", "Status"] {
            assert!(header.contains(title), "{screen}");
        }
        let first = screen.lines().find(|l| l.contains("WEB-1 ")).unwrap();
        assert!(
            first.contains(&format!("{BRANCH} feature/WEB-1")),
            "{screen}"
        );
        assert!(first.contains("● running"), "{screen}");
        assert!(first.contains("2h ago"), "{screen}");
        let second = screen.lines().find(|l| l.contains("WEB-22")).unwrap();
        assert!(second.contains("○ stopped"), "{screen}");
        assert!(screen.contains(" 2 tasks · 1 running "), "{screen}");
        assert!(screen.contains("n new   c close   x finish"), "{screen}");
    }

    #[test]
    fn the_table_grows_with_its_rows_up_to_the_window() {
        let lines = |screen: &str| screen.lines().filter(|l| l.contains('╭')).count();
        let screen = render(&tasks_state(), 100, 40);
        let top = screen
            .lines()
            .position(|l| l.contains("Last used"))
            .unwrap();
        let bottom = screen.lines().position(|l| l.contains("2 tasks")).unwrap();
        assert_eq!(bottom - top, 4, "header, rule and two rows: {screen}");
        assert_eq!(lines(&screen), 2, "search box and table: {screen}");

        let wide = render(&tasks_state(), 300, 30);
        let row = wide.lines().find(|l| l.contains("WEB-1 ")).unwrap();
        let used = row.trim().chars().count();
        assert_eq!(used, usize::from(MAX_TABLE_WIDTH), "{wide}");
    }

    #[test]
    fn long_text_is_cut_inside_its_column() {
        let mut state = State::new(vec![ProjectItem {
            name: "a-project-with-a-really-long-name".to_owned(),
            repo: "~/a/very/deep/folder/that/does/not/fit".to_owned(),
            counts: None,
        }]);
        state.handle(Key::Down);
        let screen = render(&state, 70, 30);
        let row = screen.lines().find(|l| l.contains("a-project")).unwrap();
        assert!(row.contains("…"), "{screen}");
        assert!(row.contains("not/fit"), "a folder keeps its end: {screen}");
        assert!(row.trim_end().ends_with('│'), "{screen}");
    }

    #[test]
    fn no_projects_shows_the_welcome() {
        let screen = render(&State::new(Vec::new()), 80, 30);
        assert!(screen.contains("Welcome to f0"), "{screen}");
        assert!(
            screen.contains("a  add a project from your repos"),
            "{screen}"
        );
        assert!(!screen.contains("enter"), "{screen}");
    }

    #[test]
    fn a_project_without_tasks_suggests_n() {
        let mut state = State::new(vec![project("web", Some((0, 0)))]);
        state.show_tasks("web".to_owned(), Vec::new());
        let screen = render(&state, 80, 30);
        assert!(screen.contains("No tasks yet"), "{screen}");
        assert!(screen.contains("n  start a new task"), "{screen}");
    }

    #[test]
    fn busy_spins_instead_of_the_empty_screen_or_under_the_rows() {
        let mut state = State::new(Vec::new());
        state.busy = Some("Loading projects…".to_owned());
        assert!(render(&state, 80, 30).contains("⠋ Loading projects…"));
        state.tick = 1;
        assert!(render(&state, 80, 30).contains("⠙ Loading projects…"));

        let mut state = tasks_state();
        state.busy = Some("Refreshing…".to_owned());
        let screen = render(&state, 80, 30);
        assert!(screen.contains("WEB-1"), "{screen}");
        assert!(screen.contains("⠋ Refreshing…"), "{screen}");
    }

    #[test]
    fn search_spins_then_lists_repos_or_the_folders_it_looked_in() {
        let mut state = State::new(Vec::new());
        state.show_search(vec!["~/Desktop".to_owned(), "~/code".to_owned()]);
        assert!(render(&state, 80, 30).contains("⠋ Looking for repos…"));
        state.show_repos(Vec::new());
        let screen = render(&state, 80, 30);
        assert!(screen.contains("Looked in ~/Desktop, ~/code"), "{screen}");
        assert!(screen.contains("p  type the folder of a repo"), "{screen}");
        state.show_repos(vec![RepoItem {
            path: PathBuf::from("/home/me/Desktop/app"),
            label: "~/Desktop/app".to_owned(),
        }]);
        let screen = render(&state, 80, 30);
        assert!(screen.contains(&format!("{GIT} app")), "{screen}");
        assert!(screen.contains("~/Desktop/app"), "{screen}");
    }

    #[test]
    fn the_search_box_shows_what_it_searches_then_what_is_typed() {
        let mut state = tasks_state();
        let screen = render(&state, 80, 30);
        assert!(
            screen.contains(&format!("{SEARCH}  Search tasks…")),
            "{screen}"
        );
        state.handle(Key::Char('/'));
        state.handle(Key::Char('2'));
        let screen = render(&state, 80, 30);
        assert!(screen.contains(&format!("{SEARCH}  2▏")), "{screen}");
        assert!(screen.contains(" 1 of 2 tasks"), "{screen}");
        assert!(screen.contains("enter keep search   esc clear"), "{screen}");
        state.handle(Key::Enter);
        let screen = render(&state, 80, 30);
        assert!(screen.contains(&format!("{SEARCH}  2 ")), "{screen}");
        assert!(!screen.contains("WEB-1 "), "{screen}");
    }

    #[test]
    fn prompt_and_errors_show_above_the_hints() {
        let mut state = tasks_state();
        state.handle(Key::Char('n'));
        for c in "WEB-3".chars() {
            state.handle(Key::Char(c));
        }
        let screen = render(&state, 80, 30);
        assert!(
            screen.contains("Ticket for the new task: WEB-3▏"),
            "{screen}"
        );
        assert!(screen.contains("enter create   esc cancel"), "{screen}");

        state.handle(Key::Esc);
        state.handle(Key::Char('/'));
        state.handle(Key::Char('z'));
        state.handle(Key::Enter);
        state.error("treehouse failed\nsecond line");
        let screen = render(&state, 80, 30);
        assert!(screen.contains("Nothing matches “z”"), "{screen}");
        assert!(screen.contains(&format!("{SEARCH}  z")), "{screen}");
        assert!(screen.contains("second line"), "{screen}");
    }

    #[test]
    fn finishing_a_task_asks_first() {
        let mut state = tasks_state();
        state.handle(Key::Char('x'));
        let screen = render(&state, 100, 30);
        assert!(
            screen.contains("Finish WEB-1 and return its slot?  y/n"),
            "{screen}"
        );
        assert!(screen.contains("y yes   any other key no"), "{screen}");
    }

    #[test]
    fn long_lists_scroll_to_keep_the_selection_visible() {
        let projects = (0..30)
            .map(|i| project(&format!("p{i:02}"), None))
            .collect();
        let mut state = State::new(projects);
        for _ in 0..25 {
            state.handle(Key::Down);
        }
        let screen = render(&state, 60, 24);
        assert!(screen.contains("p25"), "{screen}");
        assert!(!screen.contains("p00"), "{screen}");
        assert!(screen.contains(" 30 projects "), "{screen}");
    }

    #[test]
    fn narrow_windows_keep_only_whole_hints() {
        let screen = render(&tasks_state(), 44, 30);
        assert!(screen.contains("enter open"), "{screen}");
        assert!(!screen.contains("q quit"), "{screen}");
    }

    #[test]
    fn tiny_windows_do_not_panic() {
        for (w, h) in [(1, 1), (5, 3), (20, 6), (33, 9)] {
            render(&tasks_state(), w, h);
            render(&State::new(Vec::new()), w, h);
        }
    }

    #[test]
    fn ago_reads_like_a_person_would_say_it() {
        assert_eq!(ago(100, 90), "just now");
        assert_eq!(ago(10_000, 10_000 - 300), "5m ago");
        assert_eq!(ago(100_000, 100_000 - 7_200), "2h ago");
        assert_eq!(ago(200_000, 200_000 - 90_000), "yesterday");
        assert_eq!(ago(1_000_000, 1_000_000 - 3 * 86_400), "3d ago");
        assert_eq!(ago(10, 20), "just now", "clock skew");
    }
}
