use clap::{Parser, Subcommand, ValueEnum};
use pacmagic::{
    AdapterRegistry, Config, InstallBehavior, Operation, OperationOptions, PacMagic, PackageSource,
    PlatformDetector,
};
use std::process;

mod command_names;
mod completion;
use completion::{generate_completions, CompletionContext};

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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Shell {
    Bash,
    Fish,
    Zsh,
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
    #[arg(short = 'y', long, visible_alias = "noconfirm", global = true)]
    yes: bool,

    /// Keep confirmation prompts, overriding `--yes` and `no_confirm` in the config
    #[arg(long, global = true)]
    confirm: bool,

    /// Hide the command being executed (commands are shown by default)
    #[arg(short = 'q', long = "quiet", global = true)]
    quiet: bool,

    /// Behavior when installing packages that are already installed
    #[arg(long = "install-behavior", value_enum, global = true)]
    install_behavior: Option<InstallBehaviorArg>,

    /// Only use packages from official repositories
    #[arg(long, global = true, conflicts_with = "user")]
    official: bool,

    /// Only use packages from user repositories (e.g. the AUR)
    #[arg(long, global = true)]
    user: bool,

    /// Pass additional arguments directly to the package manager
    #[arg(long = "pm-args", global = true, num_args = 1..)]
    pm_args: Option<Vec<String>>,
}

impl Cli {
    fn no_confirm(&self, config: &Config) -> bool {
        !self.confirm && (self.yes || config.general.no_confirm)
    }

    fn package_source(&self) -> PackageSource {
        if self.official {
            PackageSource::Official
        } else if self.user {
            PackageSource::User
        } else {
            PackageSource::Any
        }
    }
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
        shell: Shell,
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

    #[command(next_help_heading = "Extra Commands")]
    /// Show detected operating system and distribution
    Distro,

    /// Check if the current distribution is a rolling release
    RollingRelease,
}

fn main() {
    let completion_env_present = std::env::var_os("COMP_LINE").is_some()
        || std::env::var_os("COMP_POINT").is_some()
        || std::env::var_os("_FISH_COMPLETE").is_some();

    if completion_env_present {
        if let Some(context) = CompletionContext::from_env() {
            let completions = generate_completions(context);
            for comp in completions {
                println!("{}", comp);
            }
        }
        return;
    }

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
        .with_no_confirm(cli.no_confirm(pm.config()))
        .with_show_commands(!cli.quiet)
        .with_source(cli.package_source())
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
        "--noconfirm",
        "--confirm",
        "-q",
        "--quiet",
        "--install-behavior",
        "--official",
        "--user",
        "--pm-args",
    ];
    let known_commands = command_names::COMMAND_NAMES;

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
            if arg.len() == 2
                && i + 1 < args.len()
                && !args[i + 1].starts_with('-')
                && !known_commands.contains(&args[i + 1].as_str())
            {
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
        .with_no_confirm(cli.no_confirm(pm.config()))
        .with_show_commands(!cli.quiet)
        .with_source(cli.package_source())
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
            completion::generate_completion_script(shell);
        }

        Commands::InitConfig { force } => {
            let config_path = Config::default_config_path()?;

            if config_path.exists() && !force {
                eprintln!("Config file already exists at: {}", config_path.display());
                eprintln!("Use --force to overwrite");
                process::exit(1);
            }

            let registry = AdapterRegistry::new();
            let pm_names: Vec<&str> = registry.all().iter().map(|pm| pm.name()).collect();
            Config::write_template_to(&config_path, &pm_names)?;
            println!("Configuration file created at: {}", config_path.display());
            println!("Edit this file to customize PacMagic behavior.");
        }

        Commands::Owns { file_or_command } => {
            pm.execute(Operation::Owns, &[&file_or_command], &options)?;
        }

        Commands::Distro => {
            println!("{}", PlatformDetector::platform_description());
        }

        Commands::RollingRelease => {
            let is_rolling = PlatformDetector::is_rolling_release();
            println!("{}", if is_rolling { "yes" } else { "no" });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_confirm(args: &[&str], config_value: bool) -> bool {
        let mut config = Config::default();
        config.general.no_confirm = config_value;
        let argv = ["pacmagic"].iter().chain(args).chain(&["list"]);
        Cli::try_parse_from(argv).unwrap().no_confirm(&config)
    }

    #[test]
    fn test_no_confirm_flags_and_config() {
        assert!(!no_confirm(&[], false));
        assert!(no_confirm(&[], true));
        assert!(no_confirm(&["-y"], false));
        assert!(no_confirm(&["--yes"], false));
        assert!(no_confirm(&["--noconfirm"], false));
        assert!(!no_confirm(&["--confirm"], true));
    }

    #[test]
    fn test_confirm_beats_yes_in_any_position() {
        assert!(!no_confirm(&["--confirm", "--yes"], false));
        assert!(!no_confirm(&["--yes", "--confirm"], false));

        let cli = Cli::try_parse_from(["pacmagic", "--confirm", "list", "--yes"]).unwrap();
        assert!(!cli.no_confirm(&Config::default()));
    }
}
