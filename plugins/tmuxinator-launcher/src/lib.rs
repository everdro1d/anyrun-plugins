use std::{
    collections::HashSet,
    env,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Directory {
    path: String,
    depth: usize,
}

#[derive(Deserialize, Clone)]
struct Config {
    prefix: String,
    terminal: Option<String>,
    tmuxinator_dir: Option<String>,
    #[serde(default)]
    directories: Vec<Directory>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: ":t".to_string(),
            terminal: None,
            tmuxinator_dir: None,
            directories: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectAction {
    Attach,
    Start,
    Create,
}

#[derive(Clone)]
struct Project {
    name: String,
    root: PathBuf,
    config: Option<PathBuf>,
    action: ProjectAction,
    is_global: bool,
}

struct State {
    config: Config,
}

#[init]
fn init(config_dir: RString) -> State {
    let config: Config =
        match fs::read_to_string(format!("{}/tmuxinator-launcher.ron", config_dir)) {
            Ok(content) => ron::from_str(&content).unwrap_or_default(),
            Err(_) => Config::default(),
        };

    State { config }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Tmuxinator Launcher".into(),
        icon: "utilities-terminal".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    // Refresh discovery each query to keep session state current.
    let projects = discover_projects(&state.config);

    if !input.starts_with(&state.config.prefix) {
        return RVec::new();
    }

    let search = input
        .strip_prefix(&state.config.prefix)
        .unwrap_or("")
        .trim()
        .to_lowercase();

    let mut matches: Vec<_> = projects
        .iter()
        .filter(|p| {
            search.is_empty()
                || p.name.to_lowercase().contains(&search)
                || p.root.to_string_lossy().to_lowercase().contains(&search)
        })
        .collect();

    matches.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    matches
        .into_iter()
        .map(|p| {
            // Description encodes locality and the project root path.
            // Handler uses this to choose between local (-p) and global start.
            let locality = if p.is_global { "global" } else { "local" };
            let path_text = if p.is_global {
                p.config
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "<unknown>".into())
            } else {
                p.root.to_string_lossy().to_string()
            };

            Match {
                title: p.name.clone().into(),
                description: ROption::RSome(
                    format!("[{}] {} {}", action_label(p.action), locality, path_text).into(),
                ),
                use_pango: false,
                icon: ROption::RNone,
                id: ROption::RNone,
            }
        })
        .collect::<Vec<_>>()
        .into()
}

#[handler]
fn handler(selection: Match, state: &State) -> HandleResult {
    let title = selection.title.as_str();

    if let ROption::RSome(desc) = selection.description.clone() {
        let desc = desc.as_str();
        // Expected format: "[action] local <path>" or "[action] global <path>"
        let action = if desc.contains("[attach]") {
            ProjectAction::Attach
        } else if desc.contains("[start]") {
            ProjectAction::Start
        } else {
            ProjectAction::Create
        };

        if let Some(rest) = desc.split("] ").nth(1) {
            if let Some(local_rest) = rest.strip_prefix("local ") {
                let root = local_rest.trim();
                let config_path = PathBuf::from(root).join(".tmuxinator.yml");
                let res = match action {
                    ProjectAction::Attach => attach(title, &state.config),
                    ProjectAction::Start => start_local(title, &config_path, &state.config),
                    ProjectAction::Create => {
                        if let Err(e) = create_basic_config(&config_path, &PathBuf::from(root), title) {
                            Err(e)
                        } else {
                            start_local(title, &config_path, &state.config)
                        }
                    }
                };
                if let Err(e) = res {
                    eprintln!("[tmuxinator-launcher] failed to run local project {title}: {e}");
                }
            } else if let Some(global_path) = rest.strip_prefix("global ") {
                // Global configs are started by project name.
                let res = match action {
                    ProjectAction::Attach => attach(title, &state.config),
                    _ => start_global(title, &state.config),
                };
                if let Err(e) = res {
                    eprintln!(
                        "[tmuxinator-launcher] failed to run global project {title} ({}): {e}",
                        global_path.trim()
                    );
                }
            }
        }
    }

    HandleResult::Close
}

// --- Discovery & actions ----------------------------------------------------

fn discover_projects(cfg: &Config) -> Vec<Project> {
    let sessions = tmux_sessions();
    let mut projects = Vec::new();

    for dir in tmuxinator_dirs(cfg) {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yml") {
                    let name = parse_project_name(&path).unwrap_or_else(|| {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string()
                    });
                    let action = determine_action(&name, true, &sessions);
                    projects.push(Project {
                        name,
                        root: dir.clone(),
                        config: Some(path),
                        action,
                        is_global: true,
                    });
                }
            }
        }
    }

    for dir in &cfg.directories {
        let root = expand_path(&dir.path);
        collect_local_projects(&root, dir.depth, &sessions, &mut projects);
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    projects.dedup_by(|a, b| a.name == b.name && a.root == b.root && a.is_global == b.is_global);
    projects
}

fn collect_local_projects(
    root: &Path,
    depth: usize,
    sessions: &HashSet<String>,
    out: &mut Vec<Project>,
) {
    if !root.exists() {
        return;
    }

    fn process_dir(dir: &Path, sessions: &HashSet<String>, out: &mut Vec<Project>) {
        let config_path = dir.join(".tmuxinator.yml");
        if config_path.exists() {
            let name = parse_project_name(&config_path)
                .or_else(|| {
                    dir.file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    config_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());
            let action = determine_action(&name, true, sessions);
            out.push(Project {
                name,
                root: dir.to_path_buf(),
                config: Some(config_path),
                action,
                is_global: false,
            });
        } else {
            let name = dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed")
                .to_string();
            let action = determine_action(&name, false, sessions);
            if action == ProjectAction::Create {
                out.push(Project {
                    name,
                    root: dir.to_path_buf(),
                    config: Some(config_path),
                    action,
                    is_global: false,
                });
            }
        }
    }

    fn walk(
        dir: &Path,
        current_depth: usize,
        depth: usize,
        sessions: &HashSet<String>,
        out: &mut Vec<Project>,
    ) {
        if current_depth == depth {
            process_dir(dir, sessions, out);
            return;
        }
        if current_depth > depth {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, current_depth + 1, depth, sessions, out);
                }
            }
        }
    }

    walk(root, 0, depth, sessions, out);
}

// Expand ~ and $VARS in paths.
fn expand_path(path: &str) -> PathBuf {
    let mut out = path.to_string();

    if out.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            out = out.replacen('~', &home, 1);
        }
    }

    // Expand $VAR
    let mut result = String::new();
    let mut chars = out.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    var.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(val) = env::var(&var) {
                result.push_str(&val);
            } else {
                result.push('$');
                result.push_str(&var);
            }
        } else {
            result.push(c);
        }
    }

    PathBuf::from(result)
}

fn default_tmuxinator_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = env::var("HOME") {
        dirs.push(PathBuf::from(format!("{home}/.config/tmuxinator")));
        dirs.push(PathBuf::from(format!("{home}/.tmuxinator")));
    }
    dirs
}

fn tmuxinator_dirs(cfg: &Config) -> Vec<PathBuf> {
    if let Some(custom) = &cfg.tmuxinator_dir {
        vec![expand_path(custom)]
    } else {
        default_tmuxinator_dirs()
    }
}

fn parse_project_name(config_path: &Path) -> Option<String> {
    let content = fs::read_to_string(config_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("project_name:") {
            return Some(rest.trim().trim_matches(['"','.']).to_string().replace(".", "-"));
        }
    }
    None
}

fn tmux_sessions() -> HashSet<String> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output();

    match output {
        Ok(out) if out.status.success() => out
            .stdout
            .split(|b| *b == b'\n')
            .filter_map(|line| std::str::from_utf8(line).ok())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect(),
        _ => HashSet::new(),
    }
}

fn determine_action(name: &str, config_exists: bool, sessions: &HashSet<String>) -> ProjectAction {
    if sessions.contains(name) {
        ProjectAction::Attach
    } else if config_exists {
        ProjectAction::Start
    } else {
        ProjectAction::Create
    }
}

fn action_label(action: ProjectAction) -> &'static str {
    match action {
        ProjectAction::Attach => "attach",
        ProjectAction::Start => "start",
        ProjectAction::Create => "create",
    }
}

// --- Launch helpers --------------------------------------------------------

fn run_in_terminal(cmd: &str, cfg: &Config) -> io::Result<()> {
    let mut candidates: Vec<String> = Vec::new();
    if let Some(term) = &cfg.terminal {
        candidates.push(term.clone());
    }
    candidates.extend([
        "x-terminal-emulator",
        "alacritty",
        "kitty",
        "wezterm",
        "gnome-terminal",
        "foot",
        "xterm",
    ]
    .into_iter()
    .map(str::to_string));

    for term in candidates {
        let spawn = Command::new(&term)
            .args(["-e", "sh", "-lc", cmd])
            .spawn();
        if spawn.is_ok() {
            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("no terminal available to run: {cmd}"),
    ))
}

fn try_switch_client(session: &str) -> bool {
    // Check if tmux is active
    if let Ok(running) = Command::new("tmux").arg("info").status() {
        if running.success() {
            // Switch the current client to the target session.
            if let Ok(status) = Command::new("tmux")
                .args(["switch-client", "-t", session])
                .status()
            {
                return status.success();
            }
        }
    }

    false
}

fn attach(session: &str, cfg: &Config) -> io::Result<()> {
    if !try_switch_client(session) {
        run_in_terminal(&format!("tmux attach-session -t {}", session), &cfg)?
    }

    Ok(())
}

// Local projects: must use `tmuxinator start -p PATH/.tmuxinator.yml`.
fn start_local(project: &str, config_path: &Path, cfg: &Config) -> io::Result<()> {
    if !try_switch_client(project) {
        run_in_terminal(&format!(
            "tmuxinator start -p {}",
            config_path.display()
        ), &cfg)?
    }

    Ok(())
}

// Global projects: `tmuxinator start PROJECT_NAME`.
fn start_global(project: &str, cfg: &Config) -> io::Result<()> {
    if !try_switch_client(project) {
        run_in_terminal(&format!("tmuxinator start {}", project), &cfg)?
    }

    Ok(())
}

fn create_basic_config(path: &Path, root: &Path, name: &str) -> io::Result<()> {
    let name = &name.replace(".", "-");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    writeln!(file, "name: {name}")?;
    writeln!(file, "root: {}", root.display())?;
    writeln!(file, "#pre_window: nix develop")?;
    writeln!(file, "startup_window: editor")?;
    writeln!(file, "windows:")?;
    writeln!(file, "  - editor:")?;
    writeln!(file, "      - nvim .")?;
    writeln!(file, "  - term:")?;
    writeln!(file, "  - git-lg: lazygit")?;
    Ok(())
}
