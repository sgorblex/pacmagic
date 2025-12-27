use clap::{Parser, Subcommand, ValueEnum};
use pacmagic::{Config, InstallBehavior, Operation, OperationOptions, PacMagic};
use std::process;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InstallBehaviorArg {
    Default,
    Upgrade,
    Reinstall,
    Skip,
}

impl From<InstallBehaviorArg> for InstallBehavior {
    fn from(arg: InstallBehaviorArg) -> Self {
        match arg {
            InstallBehaviorArg::Default => InstallBehavior::Default,
            InstallBehaviorArg::Upgrade => InstallBehavior::Upgrade,
            InstallBehaviorArg::Reinstall => InstallBehavior::Reinstall,
            InstallBehaviorArg::Skip => InstallBehavior::Skip,
        }
    }
}

#[derive(Parser)]
#[command(name = "pacmagic")]
#[command(author, version, about = "A universal package manager wrapper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

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

    /// Behavior when installing packages that are already installed
    #[arg(long = "install-behavior", value_enum, global = true)]
    install_behavior: Option<InstallBehaviorArg>,
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

    let args: Vec<String> = std::env::args().collect();
    let mut resolved_args = args.clone();

    let mut default_command_used = false;
    let mut multi_command_alias: Option<Vec<Operation>> = None;
    
    if args.len() == 1 {
        if let Ok(pm) = PacMagic::new() {
            let config = pm.config();
            if let Some(default_cmd) = &config.general.default_command {
                if let Some(ops) = config.resolve_alias_sequence(default_cmd) {
                    if ops.len() > 1 {
                        multi_command_alias = Some(ops);
                    } else if ops.len() == 1 {
                        resolved_args.push(ops[0].to_string());
                    } else {
                        resolved_args.push(default_cmd.clone());
                    }
                } else {
                    resolved_args.push(default_cmd.clone());
                }
                default_command_used = true;
            }
        }
    } else if args.len() > 1 {
        if let Ok(pm) = PacMagic::new() {
            let config = pm.config();
            if let Some(ops) = config.resolve_alias_sequence(&args[1]) {
                if ops.len() > 1 {
                    multi_command_alias = Some(ops);
                } else if ops.len() == 1 {
                    resolved_args[1] = ops[0].to_string();
                }
            } else if let Some(op) = config.resolve_alias(&args[1]) {
                resolved_args[1] = op.to_string();
            }
        }
    }

    let cli = Cli::parse_from(resolved_args);

    if let Some(ops) = multi_command_alias {
        if let Err(e) = run_multi_command(cli, ops) {
            eprintln!("Error: {}", e);
            if let Some(help) = e.help_text() {
                eprintln!("Help: {}", help);
            }
            process::exit(1);
        }
    } else {
        if let Err(e) = run(cli, default_command_used) {
            eprintln!("Error: {}", e);
            if let Some(help) = e.help_text() {
                eprintln!("Help: {}", help);
            }
            process::exit(1);
        }
    }
}

fn run_multi_command(cli: Cli, operations: Vec<Operation>) -> pacmagic::Result<()> {
    let mut pm = PacMagic::new()?;

    if let Some(pm_name) = &cli.package_manager {
        pm.config_mut().general.default_package_manager = pm_name.clone();
    }

    let options = OperationOptions::new()
        .with_verbose(cli.verbose)
        .with_dry_run(cli.dry_run)
        .with_no_confirm(cli.yes)
        .with_show_commands(!cli.quiet)
        .with_install_behavior(
            cli.install_behavior
                .map(Into::into)
                .unwrap_or(InstallBehavior::Default),
        );

    for op in operations {
        pm.execute(op, &[], &options)?;
    }

    Ok(())
}

fn run(cli: Cli, default_command_used: bool) -> pacmagic::Result<()> {
    let mut pm = PacMagic::new()?;

    if let Some(pm_name) = &cli.package_manager {
        pm.config_mut().general.default_package_manager = pm_name.clone();
    }

    let mut options = OperationOptions::new()
        .with_verbose(cli.verbose)
        .with_dry_run(cli.dry_run)
        .with_no_confirm(cli.yes)
        .with_show_commands(!cli.quiet);

    if let Some(behavior) = cli.install_behavior {
        options = options.with_install_behavior(behavior.into());
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            if default_command_used {
                if let Some(default_cmd) = &pm.config().general.default_command {
                    if let Some(ops) = pm.config().resolve_alias_sequence(default_cmd) {
                        for op in ops {
                            pm.execute(op, &[], &options)?;
                        }
                        return Ok(());
                    } else if let Some(op) = pm.config().resolve_alias(default_cmd) {
                        return execute_single_operation(&pm, op, &[], &options);
                    }
                }
            }
            eprintln!("No command specified and no default command configured.");
            eprintln!("Run 'pacmagic --help' to see available commands.");
            process::exit(1);
        }
    };

    match command {
        Commands::Install { packages } => {
            let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
            execute_single_operation(&pm, Operation::Install, &packages, &options)?;
        }

        Commands::Remove { packages } => {
            let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
            execute_single_operation(&pm, Operation::Remove, &packages, &options)?;
        }

        Commands::Update => {
            execute_single_operation(&pm, Operation::Update, &[], &options)?;
        }

        Commands::Upgrade { packages } => {
            if packages.is_empty() {
                execute_single_operation(&pm, Operation::UpgradeAll, &[], &options)?;
            } else {
                let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
                execute_single_operation(&pm, Operation::Upgrade, &packages, &options)?;
            }
        }

        Commands::Search { query } => {
            execute_single_operation(&pm, Operation::Search, &[&query], &options)?;
        }

        Commands::List => {
            execute_single_operation(&pm, Operation::List, &[], &options)?;
        }

        Commands::Info { packages } => {
            let packages: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
            execute_single_operation(&pm, Operation::Info, &packages, &options)?;
        }

        Commands::Clean => {
            execute_single_operation(&pm, Operation::Clean, &[], &options)?;
        }

        Commands::Files { package } => {
            execute_single_operation(&pm, Operation::Files, &[&package], &options)?;
        }

        Commands::Check => {
            execute_single_operation(&pm, Operation::Check, &[], &options)?;
        }

        Commands::Autoremove => {
            execute_single_operation(&pm, Operation::Autoremove, &[], &options)?;
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

fn execute_single_operation(
    pm: &PacMagic,
    operation: Operation,
    packages: &[&str],
    options: &OperationOptions,
) -> pacmagic::Result<()> {
    pm.execute(operation, packages, options)
}
