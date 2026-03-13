use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use lexicon_repo::layout::RepoLayout;

use crate::ui;

/// Which tab is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Contracts,
    Gates,
    Score,
    Api,
    Coverage,
    Help,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Dashboard, Tab::Contracts, Tab::Gates, Tab::Score, Tab::Api, Tab::Coverage, Tab::Help]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Contracts => "Contracts",
            Tab::Gates => "Gates",
            Tab::Score => "Score",
            Tab::Api => "API",
            Tab::Coverage => "Coverage",
            Tab::Help => "Help",
        }
    }

    pub fn next(&self) -> Tab {
        let tabs = Self::all();
        let idx = tabs.iter().position(|t| t == self).unwrap_or(0);
        tabs[(idx + 1) % tabs.len()]
    }

    pub fn prev(&self) -> Tab {
        let tabs = Self::all();
        let idx = tabs.iter().position(|t| t == self).unwrap_or(0);
        tabs[(idx + tabs.len() - 1) % tabs.len()]
    }
}

/// Application state for the TUI.
pub struct AppState {
    pub tab: Tab,
    pub layout: RepoLayout,
    pub contracts: Vec<String>,
    pub gate_results: Vec<lexicon_gates::result::GateResult>,
    pub score_report: Option<lexicon_scoring::engine::ScoreReport>,
    pub api_snapshot: Option<lexicon_api::schema::ApiSnapshot>,
    pub api_diff: Option<lexicon_api::diff::ApiDiff>,
    pub coverage_report: Option<lexicon_coverage::report::CoverageReport>,
    pub should_quit: bool,
    pub status_message: String,
}

impl AppState {
    pub fn new(layout: RepoLayout) -> Self {
        let (api_snapshot, api_diff) = Self::load_api_data(&layout);

        Self {
            tab: Tab::Dashboard,
            layout,
            contracts: Vec::new(),
            gate_results: Vec::new(),
            score_report: None,
            api_snapshot,
            api_diff,
            coverage_report: None,
            should_quit: false,
            status_message: String::new(),
        }
    }

    /// Try to load API snapshot and diff from the repo.
    fn load_api_data(layout: &RepoLayout) -> (Option<lexicon_api::schema::ApiSnapshot>, Option<lexicon_api::diff::ApiDiff>) {
        let api_dir = layout.root.join(".lexicon").join("api");
        let current_path = api_dir.join("current.json");
        let baseline_path = api_dir.join("baseline.json");

        let snapshot = match lexicon_api::baseline::load_baseline(&current_path) {
            Ok(snap) => Some(snap),
            Err(_) => None,
        };

        let diff = if let Some(ref current) = snapshot {
            match lexicon_api::baseline::load_baseline(&baseline_path) {
                Ok(baseline) => Some(lexicon_api::diff::diff_snapshots(&baseline, current)),
                Err(_) => None,
            }
        } else {
            None
        };

        (snapshot, diff)
    }

    /// Load data from the repo.
    pub fn refresh(&mut self) {
        // Load contracts
        self.contracts = lexicon_core::contract::contract_list(&self.layout)
            .unwrap_or_default();

        // Run verification
        match lexicon_core::verify::verify(&self.layout) {
            Ok(result) => {
                self.gate_results = result.gate_results;
                self.score_report = result.score_report;
                self.status_message = "Data refreshed".to_string();
            }
            Err(e) => {
                self.status_message = format!("Verify error: {e}");
            }
        }

        // Reload API data
        let (snapshot, diff) = Self::load_api_data(&self.layout);
        self.api_snapshot = snapshot;
        self.api_diff = diff;
    }
}

/// Run the TUI application.
pub fn run_tui(layout: RepoLayout) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(layout);
    state.refresh();

    loop {
        terminal.draw(|f| ui::draw(f, &state))?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => state.should_quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.should_quit = true;
                    }
                    KeyCode::Tab | KeyCode::Right => state.tab = state.tab.next(),
                    KeyCode::BackTab | KeyCode::Left => state.tab = state.tab.prev(),
                    KeyCode::Char('1') => state.tab = Tab::Dashboard,
                    KeyCode::Char('2') => state.tab = Tab::Contracts,
                    KeyCode::Char('3') => state.tab = Tab::Gates,
                    KeyCode::Char('4') => state.tab = Tab::Score,
                    KeyCode::Char('5') => state.tab = Tab::Api,
                    KeyCode::Char('6') => state.tab = Tab::Coverage,
                    KeyCode::Char('7') | KeyCode::Char('?') => state.tab = Tab::Help,
                    KeyCode::Char('r') => state.refresh(),
                    _ => {}
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
