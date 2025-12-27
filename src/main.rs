use clap::{Parser, Subcommand, ValueEnum};
use pacmagic::{Config, InstallBehavior, Operation, OperationOptions, PacMagic};
use std::process;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InstallBehaviorArg {
    Default,
    Upgrade,
    Reinstall,
    Skip,
    Abort,
}

impl From<InstallBehaviorArg> for InstallBehavior {
    fn from(arg: InstallBehaviorArg) -> Self {
        match arg {
            InstallBehaviorArg::Default => InstallBehavior::Default,
            InstallBehaviorArg::Upgrade => InstallBehavior::Upgrade,
            InstallBehaviorArg::Reinstall => InstallBehavior::Reinstall,
            InstallBehaviorArg::Skip => InstallBehavior::Skip,
            InstallBehaviorArg::Abort => InstallBehavior::Abort,
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

    /// Pass additional arguments directly to the package manager
    #[arg(long = "pm-args", global = true, num_args = 1..)]
    pm_args: Option<Vec<String>>,
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

    /// Find which package owns a file or command
    Owns {
        /// File path or command name
        file_or_command: String,
    },
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    let mut resolved_args = args.clone();

    let mut default_command_used = false;
    let mut multi_command_alias: Option<Vec<Operation>> = None;
    let mut package_manager_specified = false;

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
        package_manager_specified = args
            .iter()
            .any(|a| a == "-p" || a == "--package-manager" || a.starts_with("--package-manager="));

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

    let mut cli = match Cli::try_parse_from(resolved_args.clone()) {
        Ok(cli) => cli,
        Err(e) if package_manager_specified => {
            let unknown_args = extract_unknown_args(&args, &e);
            let filtered_args: Vec<String> = filter_unknown_args(&resolved_args, &unknown_args);
            match Cli::try_parse_from(filtered_args) {
                Ok(mut cli) => {
                    if !unknown_args.is_empty() {
                        if let Some(existing) = &mut cli.pm_args {
                            existing.extend(unknown_args);
                        } else {
                            cli.pm_args = Some(unknown_args);
                        }
                    }
                    cli
                }
                Err(e2) => {
                    eprintln!("{}", e2);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let unknown_args = extract_unknown_args_after_dash(&args);
    if !unknown_args.is_empty() {
        if let Some(existing) = &mut cli.pm_args {
            existing.extend(unknown_args);
        } else {
            cli.pm_args = Some(unknown_args);
        }
    }

    if let Some(ops) = multi_command_alias {
        if let Err(e) = run_multi_command(cli, ops) {
            if !e.is_command_failure() {
                eprintln!("Error: {}", e);
                if let Some(help) = e.help_text() {
                    eprintln!("Help: {}", help);
                }
            }
            process::exit(1);
        }
    } else if let Err(e) = run(cli, default_command_used) {
        if !e.is_command_failure() {
            eprintln!("Error: {}", e);
            if let Some(help) = e.help_text() {
                eprintln!("Help: {}", help);
            }
        }
        process::exit(1);
    }
}

fn run_multi_command(cli: Cli, operations: Vec<Operation>) -> pacmagic::Result<()> {
    let mut pm = PacMagic::new()?;

    if let Some(pm_name) = &cli.package_manager {
        pm.config_mut().general.default_package_manager = pm_name.clone();
    }

    let mut extra_flags = Vec::new();
    if let Some(pm_args) = &cli.pm_args {
        extra_flags.extend_from_slice(pm_args);
    }

    let mut options = OperationOptions::new()
        .with_verbose(cli.verbose)
        .with_dry_run(cli.dry_run)
        .with_no_confirm(cli.yes)
        .with_show_commands(!cli.quiet)
        .append_extra_flags(&extra_flags);

    if let Some(behavior) = cli.install_behavior {
        options = options.with_install_behavior(behavior.into());
    } else {
        options = options.with_install_behavior(pm.config().general.install_behavior);
    }

    for op in operations {
        pm.execute(op, &[], &options)?;
    }

    Ok(())
}

fn extract_unknown_args(args: &[String], _error: &clap::Error) -> Vec<String> {
    let known_global_flags = [
        "-p",
        "--package-manager",
        "-v",
        "--verbose",
        "-n",
        "--dry-run",
        "-y",
        "--yes",
        "-q",
        "--quiet",
        "--install-behavior",
        "--pm-args",
    ];

    let known_commands = [
        "install",
        "remove",
        "update",
        "upgrade",
        "search",
        "list",
        "info",
        "clean",
        "files",
        "check",
        "autoremove",
        "detect",
        "completions",
        "init-config",
        "owns",
    ];

    let mut unknown = Vec::new();
    let mut i = 0;
    let mut in_command = false;

    while i < args.len() {
        let arg = &args[i];

        if arg == "--" {
            break;
        }

        if arg == "pacmagic" || arg.ends_with("/pacmagic") {
            i += 1;
            continue;
        }

        if known_commands.contains(&arg.as_str()) {
            in_command = true;
            i += 1;
            continue;
        }

        if arg.starts_with("--package-manager=") || arg.starts_with("--pm-args") {
            i += 1;
            continue;
        }

        if arg == "-p" || arg == "--package-manager" {
            i += 1;
            if i < args.len() {
                i += 1;
            }
            continue;
        }

        if known_global_flags.contains(&arg.as_str()) {
            i += 1;
            if (arg == "-v" || arg == "-n" || arg == "-y" || arg == "-q")
                && i < args.len()
                && !args[i].starts_with('-')
            {
                i += 1;
            }
            continue;
        }

        if arg.starts_with("--install-behavior") {
            if arg.contains('=') {
                i += 1;
            } else {
                i += 1;
                if i < args.len() {
                    i += 1;
                }
            }
            continue;
        }

        if arg.starts_with('-') && !in_command {
            unknown.push(arg.clone());
            if arg.len() == 2 && i + 1 < args.len() && !args[i + 1].starts_with('-') {
                i += 1;
                unknown.push(args[i].clone());
            }
        }

        i += 1;
    }

    unknown
}

fn filter_unknown_args(args: &[String], unknown: &[String]) -> Vec<String> {
    args.iter()
        .filter(|a| !unknown.contains(a))
        .cloned()
        .collect()
}

fn extract_unknown_args_after_dash(args: &[String]) -> Vec<String> {
    let mut unknown = Vec::new();
    let mut found_dash = false;

    for arg in args {
        if arg == "--" {
            found_dash = true;
            continue;
        }
        if found_dash {
            unknown.push(arg.clone());
        }
    }

    unknown
}

fn run(cli: Cli, default_command_used: bool) -> pacmagic::Result<()> {
    let mut pm = PacMagic::new()?;

    if let Some(pm_name) = &cli.package_manager {
        pm.config_mut().general.default_package_manager = pm_name.clone();
    }

    let mut extra_flags = Vec::new();
    if let Some(pm_args) = &cli.pm_args {
        extra_flags.extend_from_slice(pm_args);
    }

    let mut options = OperationOptions::new()
        .with_verbose(cli.verbose)
        .with_dry_run(cli.dry_run)
        .with_no_confirm(cli.yes)
        .with_show_commands(!cli.quiet)
        .append_extra_flags(&extra_flags);

    if let Some(behavior) = cli.install_behavior {
        options = options.with_install_behavior(behavior.into());
    } else {
        options = options.with_install_behavior(pm.config().general.install_behavior);
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
                        return pm.execute(op, &[], &options);
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

        Commands::Owns { file_or_command } => {
            pm.execute(Operation::Owns, &[&file_or_command], &options)?;
        }
    }

    Ok(())
}
