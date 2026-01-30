use pacmagic::adapters::AdapterRegistry;
use pacmagic::config::Config;
use pacmagic::core::OperationOptions;
use std::time::{Duration, Instant};

use crate::command_names::COMMAND_NAMES;

const COMPLETION_TIMEOUT: Duration = Duration::from_millis(500);

pub struct CompletionContext {
    words: Vec<String>,
    current_word: String,
    word_index: usize,
}

impl CompletionContext {
    pub fn from_env() -> Option<Self> {
        if let (Ok(line), Ok(point)) = (std::env::var("COMP_LINE"), std::env::var("COMP_POINT")) {
            let point: usize = point.parse().ok()?;
            Self::from_line(&line, point)
        } else if let Ok(line) = std::env::var("_FISH_COMPLETE") {
            Self::from_fish_line(&line)
        } else {
            None
        }
    }

    pub fn from_line(line: &str, point: usize) -> Option<Self> {
        let mut point = point.min(line.len());
        while point > 0 && !line.is_char_boundary(point) {
            point -= 1;
        }
        let slice = &line[..point];
        let ends_with_whitespace = slice
            .chars()
            .last()
            .map(|c| c.is_whitespace())
            .unwrap_or(true);

        let words = shell_words::split(slice).ok()?;
        if words.is_empty() {
            return None;
        }

        let (current_word, word_index) = if ends_with_whitespace {
            (String::new(), words.len())
        } else {
            (
                words.last().cloned().unwrap_or_default(),
                words.len().saturating_sub(1),
            )
        };

        Some(Self {
            words,
            current_word,
            word_index,
        })
    }

    pub fn from_fish_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            return None;
        }

        let words: Vec<String> = shell_words::split(parts[0]).ok()?;
        let current_word = parts[1].to_string();
        let word_index = if current_word.is_empty() {
            words.len()
        } else {
            words.len().saturating_sub(1)
        };

        Some(Self {
            words,
            current_word,
            word_index,
        })
    }

    pub fn command(&self) -> Option<&str> {
        let mut i = 1;
        while i < self.words.len() {
            let arg = self.words[i].as_str();

            if arg == "--" {
                return None;
            }

            match arg {
                "-v" | "--verbose" | "-n" | "--dry-run" | "-y" | "--yes" | "-q" | "--quiet" => {
                    i += 1;
                    continue;
                }
                "-p" | "--package-manager" => {
                    i += 2; // flag + value
                    continue;
                }
                "--install-behavior" => {
                    i += 2; // flag + value
                    continue;
                }
                "--pm-args" => {
                    // This flag consumes the remaining args; no subcommand after it.
                    return None;
                }
                _ => {}
            }

            if arg.starts_with("--package-manager=") || arg.starts_with("--install-behavior=") {
                i += 1;
                continue;
            }

            if arg.starts_with('-') {
                i += 1;
                continue;
            }

            return Some(arg);
        }

        None
    }

    pub fn previous_word(&self) -> Option<&str> {
        if self.word_index > 0 {
            self.words.get(self.word_index - 1).map(|s| s.as_str())
        } else {
            None
        }
    }
}

pub fn generate_completions(context: CompletionContext) -> Vec<String> {
    let current_word = context.current_word.clone();
    let completions = generate_completions_impl(&context);
    completions
        .into_iter()
        .filter(|c| c.starts_with(&current_word))
        .collect()
}

fn generate_completions_impl(context: &CompletionContext) -> Vec<String> {
    let start = Instant::now();

    // After `--pm-args`, everything is intended for the underlying package manager.
    // PacMagic doesn't attempt to complete those arguments.
    if context
        .words
        .iter()
        .any(|w| w == "--pm-args" || w.starts_with("--pm-args="))
    {
        return vec![];
    }

    if context.words.len() <= 1 {
        return complete_commands();
    }

    let command = context.command();
    let previous = context.previous_word();

    match (command, previous) {
        (Some("completions"), _) => {
            vec!["bash".to_string(), "fish".to_string(), "zsh".to_string()]
        }
        (_, Some("-p")) | (_, Some("--package-manager")) => complete_package_managers(),
        (
            Some(
                "update" | "list" | "clean" | "check" | "autoremove" | "detect" | "distro"
                | "rolling-release" | "init-config",
            ),
            _,
        ) => {
            if context.word_index > 1 && start.elapsed() < COMPLETION_TIMEOUT {
                vec![]
            } else {
                complete_commands()
            }
        }
        (Some("install"), _) | (Some("remove"), _) | (Some("upgrade"), _) | (Some("info"), _) => {
            if context.word_index > 1 && start.elapsed() < COMPLETION_TIMEOUT {
                complete_packages(&context.current_word)
            } else {
                vec![]
            }
        }
        (Some("search"), _) => {
            vec![]
        }
        (Some("files"), _) => {
            if context.word_index > 1 && start.elapsed() < COMPLETION_TIMEOUT {
                complete_packages(&context.current_word)
            } else {
                vec![]
            }
        }
        (Some("owns"), _) => {
            vec![]
        }
        _ => {
            if let Some(cmd) = command {
                if Config::load()
                    .ok()
                    .and_then(|c| c.resolve_alias(cmd))
                    .is_some()
                {
                    if context.word_index > 1 && start.elapsed() < COMPLETION_TIMEOUT {
                        complete_packages(&context.current_word)
                    } else {
                        vec![]
                    }
                } else {
                    complete_commands()
                }
            } else {
                complete_commands()
            }
        }
    }
}

fn complete_commands() -> Vec<String> {
    let mut commands: Vec<String> = COMMAND_NAMES.iter().map(|c| (*c).to_string()).collect();

    if let Ok(config) = Config::load() {
        for alias in config.aliases.keys() {
            commands.push(alias.clone());
        }
    }

    commands.sort();
    commands.dedup();
    commands
}

fn complete_package_managers() -> Vec<String> {
    let registry = AdapterRegistry::new();
    registry
        .all()
        .iter()
        .map(|pm| pm.name().to_string())
        .collect()
}

fn complete_packages(prefix: &str) -> Vec<String> {
    let start = Instant::now();
    let mut packages = Vec::new();

    if let Ok(pm) = pacmagic::PacMagic::new() {
        if let Ok(package_manager) = pm.detect_package_manager() {
            let options = OperationOptions::new()
                .with_dry_run(false)
                .with_show_commands(false)
                .with_verbose(false);

            if start.elapsed() < COMPLETION_TIMEOUT {
                if let Ok(installed) = package_manager.list_installed(&options) {
                    for pkg in installed {
                        if prefix.is_empty() || pkg.name.starts_with(prefix) {
                            packages.push(pkg.name);
                        }
                        if start.elapsed() >= COMPLETION_TIMEOUT {
                            break;
                        }
                    }
                }
            }
        }
    }

    packages.sort();
    packages.dedup();
    packages
}

pub fn generate_completion_script(shell: crate::Shell) {
    match shell {
        crate::Shell::Bash => {
            println!(
                r#"# Dynamic completion for pacmagic (bash)
_pacmagic() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local completions

    # Get dynamic completions from pacmagic
    completions=$(COMP_LINE="${{COMP_LINE}}" COMP_POINT="${{COMP_POINT}}" pacmagic 2>/dev/null)

    if [[ -n "$completions" ]]; then
        local IFS=$'\n'
        COMPREPLY=($(compgen -W "$completions" -- "$cur"))
    fi
}}
complete -F _pacmagic pacmagic
"#
            );
        }
        crate::Shell::Fish => {
            println!(
                r#"# Dynamic completion for pacmagic (fish)
function __fish_pacmagic_complete
    set -l line (commandline -b)
    set -l current (commandline -ct)
    set -l tab (printf '\t')
    set -l result (env _FISH_COMPLETE="$line$tab$current" pacmagic 2>/dev/null)
    if set -q result[1]
        printf '%s\n' $result
    end
end
complete -c pacmagic -f -a '(__fish_pacmagic_complete)'
"#
            );
        }
        crate::Shell::Zsh => {
            println!(
                r#"# Dynamic completion for pacmagic (zsh)
_pacmagic() {{
    local -a completions
    local line="${{BUFFER}}"
    local point="${{CURSOR}}"

    # Get dynamic completions from pacmagic
    completions=($(COMP_LINE="$line" COMP_POINT="$point" pacmagic 2>/dev/null))

    if [[ -n "$completions" ]]; then
        compadd -a completions
    fi
}}
compdef _pacmagic pacmagic
"#
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn from_line_trailing_space_means_empty_current_word() {
        let line = "pacmagic ";
        let ctx = CompletionContext::from_line(line, line.len()).expect("context");
        assert_eq!(ctx.current_word, "");
        assert_eq!(ctx.word_index, 1);
    }

    #[test]
    fn from_line_package_manager_flag_trailing_space_completes_package_managers() {
        let line = "pacmagic -p ";
        let ctx = CompletionContext::from_line(line, line.len()).expect("context");
        let comps = generate_completions(ctx);
        assert!(comps.contains(&"apt".to_string()));
    }

    #[test]
    fn from_fish_line_empty_current_word_completes_commands() {
        let line = "pacmagic \t";
        let ctx = CompletionContext::from_fish_line(line).expect("context");
        let comps = generate_completions(ctx);
        assert!(comps.contains(&"install".to_string()));
    }

    #[test]
    fn command_skips_global_flags_before_subcommand() {
        let line = "pacmagic -p apt install fo";
        let ctx = CompletionContext::from_line(line, line.len()).expect("context");
        assert_eq!(ctx.command(), Some("install"));
    }

    #[test]
    fn pm_args_disables_pacmagic_completions() {
        let line = "pacmagic --pm-args ";
        let ctx = CompletionContext::from_line(line, line.len()).expect("context");
        let comps = generate_completions(ctx);
        assert!(comps.is_empty());
    }

    #[test]
    fn non_positional_subcommands_do_not_suggest_more_commands() {
        let line = "pacmagic update ";
        let ctx = CompletionContext::from_line(line, line.len()).expect("context");
        let comps = generate_completions(ctx);
        assert!(comps.is_empty());
    }

    #[test]
    fn from_line_non_ascii_point_does_not_panic() {
        // Place point inside a multi-byte UTF-8 character (snowman).
        let line = "pacmagic ☃";
        let ctx = CompletionContext::from_line(line, 10).expect("context");
        assert_eq!(ctx.command(), None);
    }

    #[test]
    fn from_env_bash_context() {
        let _guard = env_lock();

        let prev_comp_line = std::env::var("COMP_LINE").ok();
        let prev_comp_point = std::env::var("COMP_POINT").ok();
        let prev_fish = std::env::var("_FISH_COMPLETE").ok();

        std::env::set_var("COMP_LINE", "pacmagic ");
        std::env::set_var("COMP_POINT", "9");
        std::env::remove_var("_FISH_COMPLETE");

        let ctx = CompletionContext::from_env().expect("context");
        let comps = generate_completions(ctx);
        assert!(comps.contains(&"install".to_string()));

        match prev_comp_line {
            Some(v) => std::env::set_var("COMP_LINE", v),
            None => std::env::remove_var("COMP_LINE"),
        }
        match prev_comp_point {
            Some(v) => std::env::set_var("COMP_POINT", v),
            None => std::env::remove_var("COMP_POINT"),
        }
        match prev_fish {
            Some(v) => std::env::set_var("_FISH_COMPLETE", v),
            None => std::env::remove_var("_FISH_COMPLETE"),
        }
    }

    #[test]
    fn from_env_fish_context() {
        let _guard = env_lock();

        let prev_comp_line = std::env::var("COMP_LINE").ok();
        let prev_comp_point = std::env::var("COMP_POINT").ok();
        let prev_fish = std::env::var("_FISH_COMPLETE").ok();

        std::env::remove_var("COMP_LINE");
        std::env::remove_var("COMP_POINT");
        std::env::set_var("_FISH_COMPLETE", "pacmagic \t");

        let ctx = CompletionContext::from_env().expect("context");
        let comps = generate_completions(ctx);
        assert!(comps.contains(&"install".to_string()));

        match prev_comp_line {
            Some(v) => std::env::set_var("COMP_LINE", v),
            None => std::env::remove_var("COMP_LINE"),
        }
        match prev_comp_point {
            Some(v) => std::env::set_var("COMP_POINT", v),
            None => std::env::remove_var("COMP_POINT"),
        }
        match prev_fish {
            Some(v) => std::env::set_var("_FISH_COMPLETE", v),
            None => std::env::remove_var("_FISH_COMPLETE"),
        }
    }
}
