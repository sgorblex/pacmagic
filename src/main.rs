use clap::{Parser, Subcommand};
use pacmagic::{Config, Operation, OperationOptions, PacMagic};
use std::process;

#[derive(Parser)]
#[command(name = "pacmagic")]
#[command(author, version, about = "A universal package manager wrapper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Use a specific package manager
    #[arg(short = 'p', long, global = true)]
    package_manager: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Dry run (don't actually execute)
    #[arg(short = 'n', long, global = true)]
    dry_run: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long, global = true)]
    yes: bool,

    /// Hide the command being executed (commands are shown by default)
    #[arg(short = 'q', long = "quiet", global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install {
        /// Packages to install
        packages: Vec<String>,
    },

    /// Remove packages
    Remove {
        /// Packages to remove
        packages: Vec<String>,
    },

    /// Update package database
    Update,

    /// Upgrade specific packages
    Upgrade {
        /// Packages to upgrade (empty for all)
        packages: Vec<String>,
    },

    /// Search for packages
    Search {
        /// Search query
        query: String,
    },

    /// List installed packages
    List,

    /// Show package information
    Info {
        /// Package names
        packages: Vec<String>,
    },

    /// Clean package cache
    Clean,

    /// List files in a package
    Files {
        /// Package name
        package: String,
    },

    /// Check for issues
    Check,

    /// Remove unused dependencies
    Autoremove,

    /// Show detected package manager
    Detect,

    /// Generate shell completions
    Completions {
        /// Shell type
        shell: String,
    },

    /// Initialize configuration file
    InitConfig {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        if let Some(help) = e.help_text() {
            eprintln!("Help: {}", help);
        }
        process::exit(1);
    }
}

fn run(cli: Cli) -> pacmagic::Result<()> {
    let mut pm = PacMagic::new()?;

    // Override config with CLI flags
    if let Some(pm_name) = &cli.package_manager {
        pm.config_mut().general.default_package_manager = pm_name.clone();
    }

    // Build operation options
    let options = OperationOptions::new()
        .with_verbose(cli.verbose)
        .with_dry_run(cli.dry_run)
        .with_no_confirm(cli.yes)
        .with_show_commands(!cli.quiet); // Show commands unless --quiet is used

    match cli.command {
        Commands::Install { packages } => {
            let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
            pm.execute(Operation::Install, &packages, &options)?;
        }

        Commands::Remove { packages } => {
            let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
            pm.execute(Operation::Remove, &packages, &options)?;
        }

        Commands::Update => {
            pm.execute(Operation::Update, &[], &options)?;
        }

        Commands::Upgrade { packages } => {
            if packages.is_empty() {
                pm.execute(Operation::UpgradeAll, &[], &options)?;
            } else {
                let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
                pm.execute(Operation::Upgrade, &packages, &options)?;
            }
        }

        Commands::Search { query } => {
            pm.execute(Operation::Search, &[&query], &options)?;
        }

        Commands::List => {
            pm.execute(Operation::List, &[], &options)?;
        }

        Commands::Info { packages } => {
            let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
            pm.execute(Operation::Info, &packages, &options)?;
        }

        Commands::Clean => {
            pm.execute(Operation::Clean, &[], &options)?;
        }

        Commands::Files { package } => {
            pm.execute(Operation::Files, &[&package], &options)?;
        }

        Commands::Check => {
            pm.execute(Operation::Check, &[], &options)?;
        }

        Commands::Autoremove => {
            pm.execute(Operation::Autoremove, &[], &options)?;
        }

        Commands::Detect => {
            let detected = pm.detect_package_manager()?;
            println!("Detected package manager: {}", detected.name());
            println!("Available: {}", detected.is_available());
            println!("Priority: {}", detected.priority());
        }

        Commands::Completions { shell } => {
            println!("Shell completions for {} not yet implemented", shell);
            println!("This will be added in a future version.");
        }

        Commands::InitConfig { force } => {
            let config_path = Config::default_config_path()?;

            if config_path.exists() && !force {
                eprintln!("Config file already exists at: {}", config_path.display());
                eprintln!("Use --force to overwrite");
                process::exit(1);
            }

            let config = Config::default();
            config.save()?;
            println!("Configuration file created at: {}", config_path.display());
            println!("Edit this file to customize PacMagic behavior.");
        }
    }

    Ok(())
}
