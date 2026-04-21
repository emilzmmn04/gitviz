mod app;
mod cli;
mod git;
mod graph;
mod ui;
mod util;

use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use app::{App, DetailsTab, Mode};
use clap::Parser;
use cli::Cli;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use util::short_hash;

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
    EnsureInspect,
    Reload,
    CopyHash,
    OpenCommit,
    Quit,
}

const PAGE_SCROLL_LINES: i16 = 12;
const HALF_PAGE_SCROLL_LINES: i16 = 6;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_path: PathBuf = cli
        .repo
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    git::check_repo(&repo_path).with_context(|| {
        format!(
            "Cannot open repository at '{}'. Make sure you are inside a git repository or use --repo <path>.",
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

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    let result = run_app(&mut terminal, app, &runtime);

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
        app.clear_expired_status();
        terminal.draw(|frame| ui::view::render(frame, &mut app))?;

        if event::poll(Duration::from_millis(150))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match handle_key(&mut app, key) {
                        AppAction::None => {}
                        AppAction::EnsureInspect => ensure_selected_inspect(&mut app, runtime),
                        AppAction::Reload => {
                            if let Err(err) = reload_app(&mut app, runtime) {
                                app.set_status(format!("Reload failed: {}", err));
                            }
                        }
                        AppAction::CopyHash => copy_selected_hash(&mut app),
                        AppAction::OpenCommit => open_selected_commit(&mut app, runtime),
                        AppAction::Quit => break,
                    }
                }
                Event::Resize(_, _) => {}
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

    Ok(App::new(commits, refs, graph, runtime.colors_enabled))
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
    app.set_status("Repository reloaded");
    Ok(())
}

fn ensure_selected_inspect(app: &mut App, runtime: &RuntimeConfig) {
    if !app.should_load_selected_inspect() {
        return;
    }

    let Some(oid) = app.insert_loading_for_selected() else {
        return;
    };

    match git::load_commit_inspect_data(&runtime.repo_path, &oid) {
        Ok(data) => app.cache_inspect_ready(oid, data),
        Err(err) => app.cache_inspect_error(oid, err.to_string()),
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> AppAction {
    if app.help_open {
        return handle_help_key(app, key);
    }

    match app.mode {
        Mode::Normal => handle_normal(app, key),
        Mode::Filter => handle_filter(app, key),
    }
}

fn handle_help_key(app: &mut App, key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => app.close_help(),
        _ => {}
    }
    AppAction::None
}

fn handle_normal(app: &mut App, key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Char('?') => {
            app.toggle_help();
            AppAction::None
        }
        KeyCode::Char('j') | KeyCode::Down => selection_action(app.move_down(), app),
        KeyCode::Char('k') | KeyCode::Up => selection_action(app.move_up(), app),
        KeyCode::Char('g') | KeyCode::Home => selection_action(app.move_to_top(), app),
        KeyCode::Char('G') | KeyCode::End => selection_action(app.move_to_bottom(), app),
        KeyCode::Char('n') => selection_action(app.search_next(), app),
        KeyCode::Char('N') => selection_action(app.search_previous(), app),
        KeyCode::Enter => {
            app.toggle_details();
            AppAction::None
        }
        KeyCode::Tab => {
            app.cycle_tab_forward();
            tab_action(app)
        }
        KeyCode::BackTab => {
            app.cycle_tab_backward();
            tab_action(app)
        }
        KeyCode::PageDown => {
            app.scroll_details_lines(PAGE_SCROLL_LINES);
            AppAction::None
        }
        KeyCode::PageUp => {
            app.scroll_details_lines(-PAGE_SCROLL_LINES);
            AppAction::None
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.scroll_details_lines(HALF_PAGE_SCROLL_LINES);
            AppAction::None
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.scroll_details_lines(-HALF_PAGE_SCROLL_LINES);
            AppAction::None
        }
        KeyCode::Char('/') => {
            app.enter_filter_mode();
            AppAction::None
        }
        KeyCode::Char('r') => AppAction::Reload,
        KeyCode::Char('y') => AppAction::CopyHash,
        KeyCode::Char('o') => AppAction::OpenCommit,
        KeyCode::Char('q') => AppAction::Quit,
        _ => AppAction::None,
    }
}

fn selection_action(changed: bool, app: &App) -> AppAction {
    if changed && !matches!(app.active_tab, DetailsTab::Summary) {
        AppAction::EnsureInspect
    } else {
        AppAction::None
    }
}

fn tab_action(app: &App) -> AppAction {
    if matches!(app.active_tab, DetailsTab::Summary) {
        AppAction::None
    } else {
        AppAction::EnsureInspect
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) -> AppAction {
    match key.code {
        KeyCode::Char('?') => {
            app.toggle_help();
            AppAction::None
        }
        KeyCode::Esc => {
            app.exit_filter_mode();
            AppAction::None
        }
        KeyCode::Backspace => {
            app.filter_pop();
            AppAction::None
        }
        KeyCode::Enter => {
            app.confirm_filter();
            AppAction::None
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.filter_push(c);
            AppAction::None
        }
        _ => AppAction::None,
    }
}

fn copy_selected_hash(app: &mut App) {
    let Some(commit) = app.selected_commit() else {
        app.set_status("No commit selected");
        return;
    };

    match copy_to_clipboard(&commit.oid) {
        Ok(()) => app.set_status(format!("Copied {} to clipboard", short_hash(&commit.oid))),
        Err(_) => app.set_status("Copy failed: no supported clipboard command found"),
    }
}

fn open_selected_commit(app: &mut App, runtime: &RuntimeConfig) {
    let Some(commit) = app.selected_commit() else {
        app.set_status("No commit selected");
        return;
    };

    let Some(url) = git::github_commit_url(&runtime.repo_path, &commit.oid) else {
        app.set_status("Open unavailable: origin is not a supported GitHub remote");
        return;
    };

    match open_url(&url) {
        Ok(()) => app.set_status(format!("Opened {} in GitHub", short_hash(&commit.oid))),
        Err(_) => app.set_status("Open failed: no supported browser command found"),
    }
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        pipe_to_command("pbcopy", &[], text)?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let candidates: [(&str, &[&str]); 3] = [
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ];

        for (command, args) in candidates {
            if pipe_to_command(command, args, text).is_ok() {
                return Ok(());
            }
        }

        anyhow::bail!("no supported clipboard command found");
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("clipboard unsupported on this platform")
}

fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("failed to run open")?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("failed to run xdg-open")?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("browser open unsupported on this platform")
}

fn pipe_to_command(command: &str, args: &[&str], input: &str) -> Result<()> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to spawn {}", command))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("{} exited with {}", command, status)
    }
}
