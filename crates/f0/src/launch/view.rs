//! Draws the launcher's `State`: the f0 wordmark, the current list as a grid of cards,
//! and one quiet line of key hints at the bottom. Only the selected card uses the
//! accent color.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};

use super::state::{PromptKind, Screen, State};

/// The wordmark, drawn when the window is tall enough for it.
const LOGO: [&str; 5] = [
    "██████  ██████",
    "██      ██  ██",
    "█████   ██  ██",
    "██      ██  ██",
    "██      ██████",
];
/// Below this height the wordmark shrinks to one line, leaving room for the cards.
const LOGO_MIN_HEIGHT: u16 = 26;

/// Card size, borders included, and the space between cards.
pub const CARD_WIDTH: u16 = 32;
const CARD_HEIGHT: u16 = 5;
const CARD_GAP: u16 = 2;
/// Wider rows get hard to scan.
const MAX_COLUMNS: u16 = 4;

/// Braille frames of the spinner.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const RUNNING: &str = "●";
const STOPPED: &str = "○";
/// Nerd Font glyphs, from the font f0's Ghostty config sets.
const BRANCH: &str = "\u{e0a0}";
const GIT: &str = "\u{e702}";

/// The f0 palette for the launcher, light or dark like the bundled configs.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Painted under everything, so the launcher keeps its mode even when the terminal
    /// was not told about it.
    pub background: Color,
    /// Only the selected card uses it.
    pub accent: Color,
    pub text: Color,
    /// Labels and metadata.
    pub muted: Color,
    /// Borders, key hints and the least important metadata.
    pub faint: Color,
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
                running: Color::Rgb(0x3d, 0x7a, 0x1f),
                error: Color::Rgb(0xc7, 0x2e, 0x2e),
            }
        }
    }
}

/// How many cards fit side by side in `width`.
pub fn columns_for(width: u16) -> usize {
    let fit = (width + CARD_GAP) / (CARD_WIDTH + CARD_GAP);
    usize::from(fit.clamp(1, MAX_COLUMNS))
}

/// The width the cards get from a window `width` wide: what `draw` lays them out in.
pub fn grid_width(width: u16) -> u16 {
    width.saturating_sub(4).max(1)
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

/// One card's three lines.
struct Card {
    title: String,
    second: Vec<Span<'static>>,
    third: Vec<Span<'static>>,
    /// The line, 2 or 3, that holds a folder, which is cut at the start to keep its end.
    path_line: Option<u8>,
}

/// Draws `state` on the whole frame. `now` dates the last use of the tasks.
pub fn draw(frame: &mut Frame, state: &State, palette: Palette, now: u64) {
    let area = frame.area();
    frame.render_widget(
        Block::new().style(Style::new().bg(palette.background)),
        area,
    );
    let column = Rect {
        x: area.x + 2.min(area.width),
        width: grid_width(area.width).min(area.width),
        ..area
    };
    let tall = area.height >= LOGO_MIN_HEIGHT;
    let header_height = if tall { LOGO.len() as u16 + 4 } else { 3 };
    let bottom_lines = bottom_text(state, palette).len().min(6) as u16;
    let [header, list, bottom, _, hints] = Layout::vertical([
        Constraint::Length(header_height),
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
    draw_cards(frame, state, palette, now, list);
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

fn draw_cards(frame: &mut Frame, state: &State, palette: Palette, now: u64, area: Rect) {
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

    let columns = columns_for(area.width);
    let card_width = CARD_WIDTH.min(area.width);
    // A single short row is centered on the cards it has.
    let used = columns.min(visible.len()) as u16;
    let grid = used * card_width + (used - 1) * CARD_GAP;
    let left = area.x + area.width.saturating_sub(grid) / 2;
    let rows_fit = usize::from((area.height / CARD_HEIGHT).max(1));
    let first_row = (state.selected / columns).saturating_sub(rows_fit - 1);

    for (position, &index) in visible.iter().enumerate().skip(first_row * columns) {
        let (row, col) = (position / columns - first_row, position % columns);
        if row >= rows_fit {
            break;
        }
        let rect = Rect {
            x: left + col as u16 * (card_width + CARD_GAP),
            y: area.y + row as u16 * CARD_HEIGHT,
            width: card_width,
            height: CARD_HEIGHT,
        }
        .intersection(area);
        if rect.is_empty() {
            continue;
        }
        let card = card(state, index, palette, now);
        draw_card(frame, &card, position == state.selected, palette, rect);
    }
}

fn card(state: &State, index: usize, palette: Palette, now: u64) -> Card {
    let muted = |text: String| Span::styled(text, Style::new().fg(palette.muted));
    let faint = |text: String| Span::styled(text, Style::new().fg(palette.faint));
    let running = || Span::styled(RUNNING, Style::new().fg(palette.running));
    let stopped = || Span::styled(STOPPED, Style::new().fg(palette.faint));
    match &state.screen {
        Screen::Projects => {
            let project = &state.projects[index];
            let second = match project.counts {
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
            Card {
                title: project.name.clone(),
                second,
                third: vec![faint(project.repo.clone())],
                path_line: Some(3),
            }
        }
        Screen::Tasks { .. } | Screen::Sessions => {
            let task = &state.tasks[index];
            let status = match (&task.session, task.activity) {
                (Some(_), Some(then)) => vec![running(), muted(format!(" {}", ago(now, then)))],
                (Some(_), None) => vec![running(), muted(" running".to_owned())],
                (None, _) => vec![stopped(), faint(" stopped".to_owned())],
            };
            if task.branch.is_empty() {
                Card {
                    title: task.task.clone(),
                    second: status,
                    third: Vec::new(),
                    path_line: None,
                }
            } else {
                Card {
                    title: task.task.clone(),
                    second: vec![muted(format!("{BRANCH} {}", task.branch))],
                    third: status,
                    path_line: None,
                }
            }
        }
        Screen::AddRepo { .. } => {
            let repo = &state.repos[index];
            let name = repo
                .path
                .file_name()
                .map_or_else(|| repo.label.clone(), |n| n.to_string_lossy().into_owned());
            Card {
                title: format!("{GIT} {name}"),
                second: vec![faint(repo.label.clone())],
                third: Vec::new(),
                path_line: Some(2),
            }
        }
    }
}

fn draw_card(frame: &mut Frame, card: &Card, selected: bool, palette: Palette, rect: Rect) {
    let (border, border_color, title) = if selected {
        (
            BorderType::Thick,
            palette.accent,
            Style::new().fg(palette.accent).add_modifier(Modifier::BOLD),
        )
    } else {
        (
            BorderType::Rounded,
            palette.faint,
            Style::new().fg(palette.text).add_modifier(Modifier::BOLD),
        )
    };
    let inner = usize::from(rect.width.saturating_sub(4));
    let fit = |spans: &[Span<'static>], line: u8| match spans {
        [path] if card.path_line == Some(line) => {
            vec![Span::styled(
                truncate_start(&path.content, inner),
                path.style,
            )]
        }
        _ => truncate_spans(spans, inner),
    };
    let lines = vec![
        Line::from(Span::styled(truncate(&card.title, inner), title)),
        Line::from(fit(&card.second, 2)),
        Line::from(fit(&card.third, 3)),
    ];
    let block = Block::bordered()
        .border_type(border)
        .border_style(Style::new().fg(border_color))
        .padding(ratatui::widgets::Padding::horizontal(1));
    frame.render_widget(Paragraph::new(lines).block(block), rect);
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

/// The prompt or filter, a spinner when busy over a list, and the message.
fn bottom_text(state: &State, palette: Palette) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(prompt) = &state.prompt
        && prompt.kind == PromptKind::Confirm
    {
        lines.push(Line::from(vec![
            Span::styled(prompt.text.clone(), Style::new().fg(palette.text)),
            Span::styled("  y/n", Style::new().fg(palette.accent)),
        ]));
    } else if let Some(prompt) = &state.prompt {
        let label = match prompt.kind {
            PromptKind::Filter => "/ ",
            PromptKind::Ticket => "Ticket for the new task: ",
            PromptKind::Path | PromptKind::Confirm => "Repo folder: ",
        };
        lines.push(Line::from(vec![
            Span::styled(label, Style::new().fg(palette.muted)),
            Span::styled(prompt.text.clone(), Style::new().fg(palette.text)),
            Span::styled("▏", Style::new().fg(palette.accent)),
        ]));
    } else if !state.filter.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("/ {}", state.filter),
            Style::new().fg(palette.muted),
        )));
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
            PromptKind::Filter => vec![("enter", "keep filter"), ("esc", "clear")],
            PromptKind::Ticket => vec![("enter", "create"), ("esc", "cancel")],
            PromptKind::Path => vec![("enter", "add"), ("esc", "cancel")],
            PromptKind::Confirm => vec![("y", "yes"), ("any other key", "no")],
        };
    }
    let any = !state.visible().is_empty();
    let mut hints = Vec::new();
    if any {
        hints.push(("←↑↓→", "move"));
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
    hints.extend([("/", "filter"), ("q", "quit")]);
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
    fn project_cards_show_counts_and_repo() {
        let state = State::new(vec![
            project("api", Some((2, 1))),
            project("web", Some((0, 0))),
            project("cli", None),
        ]);
        let screen = render(&state, 120, 30);
        assert!(screen.contains("● 2 running  ○ 1 stopped"), "{screen}");
        assert!(screen.contains("no tasks"), "{screen}");
        assert!(screen.contains("slots unknown"), "{screen}");
        assert!(screen.contains("~/src/api"), "{screen}");
        assert!(screen.contains("┏"), "the selected card is thick: {screen}");
        assert!(screen.contains("╭"), "the others are rounded: {screen}");
        assert!(
            screen.contains("enter tasks   a add   e edit   d doctor   / filter   q quit"),
            "{screen}"
        );
    }

    #[test]
    fn only_the_selected_card_uses_the_accent() {
        let state = State::new(vec![project("api", None), project("web", None)]);
        let palette = Palette::new(false);
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal
            .draw(|frame| draw(frame, &state, palette, NOW))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let accent: Vec<(u16, u16)> = (0..buffer.area.height)
            .flat_map(|y| (0..buffer.area.width).map(move |x| (x, y)))
            .filter(|&(x, y)| buffer[(x, y)].fg == palette.accent)
            .collect();
        let max_x = accent.iter().map(|(x, _)| *x).max().unwrap();
        let min_x = accent.iter().map(|(x, _)| *x).min().unwrap();
        assert!(max_x - min_x < CARD_WIDTH, "accent spans one card only");
    }

    #[test]
    fn task_cards_show_branch_and_status() {
        let screen = render(&tasks_state(), 100, 30);
        assert!(screen.contains("web  ·  Tasks"), "{screen}");
        assert!(screen.contains("WEB-1"), "{screen}");
        assert!(
            screen.contains(&format!("{BRANCH} feature/WEB-1")),
            "{screen}"
        );
        assert!(screen.contains("● 2h ago"), "{screen}");
        assert!(screen.contains("○ stopped"), "{screen}");
        assert!(screen.contains("n new   c close   x finish"), "{screen}");
    }

    #[test]
    fn cards_wrap_into_rows_that_fit_the_window() {
        assert_eq!(columns_for(10), 1);
        assert_eq!(columns_for(CARD_WIDTH), 1);
        assert_eq!(columns_for(2 * CARD_WIDTH + CARD_GAP), 2);
        assert_eq!(columns_for(500), MAX_COLUMNS as usize);
        let wide = render(&tasks_state(), 100, 30);
        let line = wide.lines().find(|l| l.contains("WEB-1")).unwrap();
        assert!(line.contains("WEB-22"), "side by side: {wide}");
        let narrow = render(&tasks_state(), 40, 30);
        let line = narrow.lines().find(|l| l.contains("WEB-1")).unwrap();
        assert!(!line.contains("WEB-22"), "stacked: {narrow}");
    }

    #[test]
    fn a_short_row_is_centered() {
        let screen = render(&tasks_state(), 120, 30);
        let line = screen.lines().find(|l| l.contains("WEB-1")).unwrap();
        let left = line.find('┃').unwrap();
        let right = line.len() - line.rfind('│').unwrap() - '│'.len_utf8();
        assert!(left.abs_diff(right) <= 2, "{screen}");
    }

    #[test]
    fn long_text_is_cut_inside_the_card() {
        let mut state = State::new(vec![ProjectItem {
            name: "a-project-with-a-really-long-name".to_owned(),
            repo: "~/a/very/deep/folder/that/does/not/fit".to_owned(),
            counts: None,
        }]);
        state.handle(Key::Down);
        let screen = render(&state, 60, 30);
        assert!(screen.contains("a-project-with-a-really-lon…"), "{screen}");
        assert!(screen.contains("…ep/folder/that/does/not/fit"), "{screen}");
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
    fn busy_spins_instead_of_the_empty_screen_or_under_the_cards() {
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
    fn prompt_filter_and_errors_show_above_the_hints() {
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
        assert!(screen.contains("/ z"), "{screen}");
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
        state.set_columns(1);
        for _ in 0..25 {
            state.handle(Key::Down);
        }
        let screen = render(&state, 40, 24);
        assert!(screen.contains("p25"), "{screen}");
        assert!(!screen.contains("p00"), "{screen}");
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
