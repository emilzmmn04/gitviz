mod app;
mod cli;
mod git;
mod graph;
mod ui;
mod util;

use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, Mode};
use cli::Cli;

#[derive(Clone)]
struct RuntimeConfig {
    repo_path: PathBuf,
    max: usize,
    all: bool,
    exclude_reachable_from: Option<String>,
    colors_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppAction {
    None,
    Reload,
    Quit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_path: PathBuf = cli
        .repo
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Validate repository
    git::check_repo(&repo_path).with_context(|| {
        format!(
            "Cannot open repository at '{}'. \
             Make sure you are inside a git repository or use --repo <path>.",
            repo_path.display()
        )
    })?;

    let runtime = RuntimeConfig {
        repo_path,
        max: cli.max,
        all: cli.all,
        exclude_reachable_from: cli.exclude_reachable_from,
        colors_enabled: !cli.no_color,
    };

    let app = load_app(&runtime)?;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Run the TUI loop — restore terminal on any error
    let result = run_app(&mut terminal, app, &runtime);

    // Always restore terminal
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
    runtime: &RuntimeConfig,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::view::render(f, &app))?;

        if event::poll(Duration::from_millis(150))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match handle_key(&mut app, key.code) {
                        AppAction::None => {}
                        AppAction::Quit => break,
                        AppAction::Reload => reload_app(&mut app, runtime)?,
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal resize — just re-render on next loop iteration.
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn load_app(runtime: &RuntimeConfig) -> Result<App> {
    eprintln!("Loading commits from {} …", runtime.repo_path.display());

    let commits = git::load_commits(
        &runtime.repo_path,
        runtime.max,
        runtime.all,
        runtime.exclude_reachable_from.as_deref(),
    )
    .context("Failed to load commits")?;

    let refs = git::load_refs(&runtime.repo_path).context("Failed to load refs")?;
    let graph = graph::compute_layout(&commits);

    Ok(App::new(
        commits,
        refs,
        graph,
        runtime.colors_enabled,
    ))
}

fn reload_app(app: &mut App, runtime: &RuntimeConfig) -> Result<()> {
    let commits = git::load_commits(
        &runtime.repo_path,
        runtime.max,
        runtime.all,
        runtime.exclude_reachable_from.as_deref(),
    )
    .context("Failed to reload commits")?;
    let refs = git::load_refs(&runtime.repo_path).context("Failed to reload refs")?;
    let graph = graph::compute_layout(&commits);
    app.replace_data(commits, refs, graph);
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) -> AppAction {
    match app.mode {
        Mode::Normal => handle_normal(app, code),
        Mode::Filter => handle_filter(app, code),
    }
}

fn handle_normal(app: &mut App, code: KeyCode) -> AppAction {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('g') | KeyCode::Home => app.move_to_top(),
        KeyCode::Char('G') | KeyCode::End => app.move_to_bottom(),
        KeyCode::Enter => app.toggle_details(),
        KeyCode::Char('/') => app.enter_filter_mode(),
        KeyCode::Char('r') => return AppAction::Reload,
        KeyCode::Char('q') => return AppAction::Quit,
        _ => {}
    }
    AppAction::None
}

fn handle_filter(app: &mut App, code: KeyCode) -> AppAction {
    match code {
        KeyCode::Esc => app.exit_filter_mode(),
        KeyCode::Backspace => app.filter_pop(),
        KeyCode::Char(c) => app.filter_push(c),
        KeyCode::Enter => {
            // Confirm filter, go back to normal mode but keep filter active
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    AppAction::None
}
