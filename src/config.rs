use super::*;
use commands::Command;

pub mod commands;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(name = "CI manager")]
#[command(bin_name = "ci-manager", version, propagate_version = true, author, styles = config_styles())]
#[command(about = "Manage CI")]
pub struct Config {
    #[command(subcommand)]
    command: Option<Command>,
    /// Generate completion scripts for the specified shell
    #[arg(long, global = true, value_hint = ValueHint::Other, name = "SHELL")]
    completions: Option<clap_complete::Shell>,
    /// Verbosity level (0-4)
    #[arg(short, long, global = true, default_value_t = 2)]
    verbosity: u8,
    /// Debug flag to run through a scenario without making changes
    #[arg(long, default_value_t = false, global = true)]
    dry_run: bool,
    /// Override the CI provider detection and assume the specified provider
    #[arg(value_enum, long, global = true)]
    ci: Option<CIProvider>,
    /// Trim the prefix timestamp from the log output
    #[arg(long, global = true, default_value_t = false)]
    trim_timestamp: bool,
}

impl Config {
    /// Get a reference to the global config
    pub fn global() -> &'static Config {
        CONFIG.get().expect("Config is not initialized")
    }

    /// Get the subcommand
    pub fn subcmd(&self) -> &Command {
        if let Some(subcmd) = &self.command {
            subcmd
        } else {
            log::error!("Subcommand required! use `--help` for more information");
            std::process::exit(1);
        }
    }

    /// Generate completion scripts for the specified shell, returns true if a shell was specified
    /// meaning the user wants to generate a completion script
    pub fn generate_completion_script(&self) -> bool {
        match self.completions {
            Some(shell) => {
                generate_completion_script(shell);
                true
            }
            None => false,
        }
    }

    /// Get the verbosity level
    pub fn verbosity(&self) -> u8 {
        self.verbosity
    }

    /// Get the dry run flag
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    /// Get the CI provider override
    pub fn no_ci(&self) -> Option<CIProvider> {
        self.ci
    }

    /// Get the trim timestamp flag
    pub fn trim_timestamp(&self) -> bool {
        self.trim_timestamp
    }
}

/// Initialize the CLI configuration
pub fn init() -> Result<()> {
    let config = Config::parse();
    CONFIG.set(config).expect("Config is already initialized");

    use stderrlog::LogLevelNum;
    let log_level = match Config::global().verbosity() {
        0 => LogLevelNum::Error,
        1 => LogLevelNum::Warn,
        2 => LogLevelNum::Info,
        3 => LogLevelNum::Debug,
        4 => LogLevelNum::Trace,
        _ => {
            eprintln!("Invalid verbosity level: {}", Config::global().verbosity());
            eprintln!("Using highest verbosity level: Trace");
            LogLevelNum::Trace
        }
    };
    stderrlog::new().verbosity(log_level).quiet(false).init()?;
    if Config::global().dry_run() {
        log::warn!("Running in dry-run mode. No writes/changes will be made");
    }

    Ok(())
}

// Styles for the help messages in the CLI
fn config_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Blue.on_default())
}

/// Generate completion scripts for the specified shell
fn generate_completion_script(shell: clap_complete::Shell) {
    log::info!("Generating completion script for {shell:?}");
    clap_complete::generate(
        shell,
        &mut <Config as clap::CommandFactory>::command(),
        "ci-manager",
        &mut std::io::stdout(),
    );
}
