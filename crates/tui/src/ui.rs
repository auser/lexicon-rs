use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;

use lexicon_gates::result::GateOutcome;
use lexicon_scoring::engine::Verdict;

use crate::app::{AppState, Tab};

pub fn draw(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(0),   // main content
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    draw_tabs(f, state, chunks[0]);

    match state.tab {
        Tab::Dashboard => draw_dashboard(f, state, chunks[1]),
        Tab::Contracts => draw_contracts(f, state, chunks[1]),
        Tab::Gates => draw_gates(f, state, chunks[1]),
        Tab::Score => draw_score(f, state, chunks[1]),
        Tab::Help => draw_help(f, chunks[1]),
    }

    draw_status_bar(f, state, chunks[2]);
}

fn draw_tabs(f: &mut Frame, state: &AppState, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == state.tab {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(t.label(), style))
        })
        .collect();

    let idx = Tab::all().iter().position(|t| *t == state.tab).unwrap_or(0);
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" lexicon "))
        .select(idx)
        .highlight_style(Style::default().fg(Color::Cyan));

    f.render_widget(tabs, area);
}

fn draw_dashboard(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: summary
    let mut lines = vec![
        Line::from(Span::styled(
            "Repository Health",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("  Root: {}", state.layout.root.display())),
        Line::from(format!("  Contracts: {}", state.contracts.len())),
        Line::from(format!("  Gates: {}", state.gate_results.len())),
    ];

    if let Some(ref report) = state.score_report {
        let (color, label) = match report.verdict {
            Verdict::Pass => (Color::Green, "PASS"),
            Verdict::Warn => (Color::Yellow, "WARN"),
            Verdict::Fail => (Color::Red, "FAIL"),
        };
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  Score: "),
            Span::styled(
                format!("{:.1}% {label}", report.total_score * 100.0),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let summary = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));
    f.render_widget(summary, chunks[0]);

    // Right: recent gates
    let gate_items: Vec<ListItem> = state
        .gate_results
        .iter()
        .map(|gr| {
            let (icon, color) = match gr.outcome {
                GateOutcome::Pass => ("✓", Color::Green),
                GateOutcome::Fail => ("✗", Color::Red),
                GateOutcome::Skip => ("⊘", Color::Yellow),
                GateOutcome::Error => ("!", Color::Red),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(color)),
                Span::raw(format!("{} ({}ms)", gr.gate_id, gr.duration_ms)),
            ]))
        })
        .collect();

    let gates_list = List::new(gate_items)
        .block(Block::default().borders(Borders::ALL).title(" Gates "));
    f.render_widget(gates_list, chunks[1]);
}

fn draw_contracts(f: &mut Frame, state: &AppState, area: Rect) {
    if state.contracts.is_empty() {
        let msg = Paragraph::new("No contracts found. Run `lexicon contract new` to create one.")
            .block(Block::default().borders(Borders::ALL).title(" Contracts "));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = state
        .contracts
        .iter()
        .map(|id| ListItem::new(format!("  {id}")))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Contracts "));
    f.render_widget(list, area);
}

fn draw_gates(f: &mut Frame, state: &AppState, area: Rect) {
    if state.gate_results.is_empty() {
        let msg = Paragraph::new("No gate results. Run `lexicon verify` first.")
            .block(Block::default().borders(Borders::ALL).title(" Gates "));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = state
        .gate_results
        .iter()
        .map(|gr| {
            let (icon, color) = match gr.outcome {
                GateOutcome::Pass => ("✓", Color::Green),
                GateOutcome::Fail => ("✗", Color::Red),
                GateOutcome::Skip => ("⊘", Color::Yellow),
                GateOutcome::Error => ("!", Color::Red),
            };
            let detail = if !gr.stderr.is_empty() {
                format!(" — {}", gr.stderr.lines().next().unwrap_or(""))
            } else {
                String::new()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(color)),
                Span::raw(format!(
                    "{} ({}ms){}",
                    gr.gate_id, gr.duration_ms, detail
                )),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Gates "));
    f.render_widget(list, area);
}

fn draw_score(f: &mut Frame, state: &AppState, area: Rect) {
    let report = match state.score_report {
        Some(ref r) => r,
        None => {
            let msg = Paragraph::new("No score data. Run `lexicon score init` and `lexicon verify`.")
                .block(Block::default().borders(Borders::ALL).title(" Score "));
            f.render_widget(msg, area);
            return;
        }
    };

    let (color, label) = match report.verdict {
        Verdict::Pass => (Color::Green, "PASS"),
        Verdict::Warn => (Color::Yellow, "WARN"),
        Verdict::Fail => (Color::Red, "FAIL"),
    };

    let mut lines = vec![
        Line::from(vec![
            Span::raw("Total Score: "),
            Span::styled(
                format!("{:.1}% ({label})", report.total_score * 100.0),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Dimensions:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];

    for dim in &report.dimensions {
        let dim_color = if dim.passed { Color::Green } else { Color::Red };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:.0}% ", dim.value * 100.0),
                Style::default().fg(dim_color),
            ),
            Span::raw(&dim.dimension_id),
            Span::styled(
                format!(" — {}", dim.explanation),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Score Breakdown "));
    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Tab / → / ←    Switch tabs"),
        Line::from("  1-5            Jump to tab"),
        Line::from("  r              Refresh data"),
        Line::from("  q / Esc        Quit"),
        Line::from("  Ctrl+C         Force quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Commands",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  lexicon init           Initialize lexicon"),
        Line::from("  lexicon contract new   Create a contract"),
        Line::from("  lexicon score init     Initialize scoring"),
        Line::from("  lexicon gate init      Initialize gates"),
        Line::from("  lexicon verify         Run verification"),
        Line::from("  lexicon sync claude    Sync CLAUDE.md"),
        Line::from("  lexicon doctor         Check repo health"),
    ];

    let help = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, area);
}

fn draw_status_bar(f: &mut Frame, state: &AppState, area: Rect) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" lexicon ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(format!(" {} ", state.status_message)),
        Span::styled(
            " q:quit  r:refresh  Tab:switch ",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(status, area);
}
