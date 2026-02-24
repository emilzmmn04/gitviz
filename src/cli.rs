use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "gitviz",
    version,
    about = "Terminal Git repository visualizer",
    long_about = "Visualize git commit history as an interactive TUI graph.\n\
                  Navigate with j/k or arrow keys. Press q to quit."
)]
pub struct Cli {
    /// Show all branches (not just HEAD)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub all: bool,

    /// Maximum number of commits to display
    #[arg(long, short = 'n', default_value_t = 200)]
    pub max: usize,

    /// Exclude commits reachable from this revision (e.g. HEAD~500)
    #[arg(long)]
    pub since: Option<String>,

    /// Path to the git repository (default: current directory)
    #[arg(long)]
    pub repo: Option<String>,

    /// Disable colors
    #[arg(long)]
    pub no_color: bool,
}
