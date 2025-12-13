use std::{fs, process::Command};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin:: *;
use serde:: Deserialize;

#[derive(Deserialize)]
struct Config {
    prefix: String,
    bookmarks_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: ":b".to_string(),
            bookmarks_file: "~/bookmarks.txt".to_string(),
        }
    }
}

#[derive(Clone)]
struct Bookmark {
    tag: String,
    name: String,
    url: String,
}

struct State {
    config: Config,
    bookmarks: Vec<Bookmark>,
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

fn parse_bookmarks(content: &str) -> Vec<Bookmark> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            // Parse:   [TAG] <NAME>, <URL>
            let tag_end = line.find(']')?;
            let tag = line. get(1..tag_end)?.trim().to_string();

            let rest = line.get(tag_end + 1..)?.trim();
            let (name, url) = rest.split_once(',')?;

            Some(Bookmark {
                tag,
                name: name.trim().to_string(),
                url: url.trim().to_string(),
            })
        })
        .collect()
}

#[init]
fn init(config_dir: RString) -> State {
    let config: Config =
        match fs::read_to_string(format!("{}/bookmarks-launcher.ron", config_dir)) {
            Ok(content) => ron::from_str(&content).unwrap_or_default(),
            Err(_) => Config::default(),
        };

    let bookmarks_path = expand_tilde(&config. bookmarks_file);
    let bookmarks = match fs::read_to_string(&bookmarks_path) {
        Ok(content) => parse_bookmarks(&content),
        Err(e) => {
            eprintln!("[bookmarks-launcher] Failed to read bookmarks file: {}", e);
            Vec::new()
        }
    };

    State { config, bookmarks }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Bookmarks Launcher".into(),
        icon: "application-x-executable".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    // Check for prefix
    if !input.starts_with(&state.config.prefix) {
        return RVec::new();
    }

    let search = input
        .strip_prefix(&state.config.prefix)
        .unwrap_or("")
        .trim()
        .to_lowercase();

    let mut matches: Vec<_> = state
        .bookmarks
        .iter()
        .filter(|b| {
            // Show all if search is empty, otherwise filter by name, tag, or url
            search.is_empty()
                || b.tag. to_lowercase().contains(&search)
                || b.name.to_lowercase().contains(&search)
                || b.url.to_lowercase().contains(&search)
        })
        .collect();

    // Sort by tag first, then by name within each tag group
    matches.sort_by(|a, b| {
        a.tag
            .to_lowercase()
            .cmp(&b.tag. to_lowercase())
            .then_with(|| a.name. to_lowercase().cmp(&b.name.to_lowercase()))
    });

    matches
        .into_iter()
        .map(|b| Match {
            title: b.name.clone().into(),
            description: ROption::RSome(format!("[{}] {}", b.tag, b.url).into()),
            use_pango: false,
            icon: ROption::RNone,
            id: ROption::RNone,
        })
        .collect::<Vec<_>>()
        .into()
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    if let ROption::RSome(desc) = selection.description {
        // Extract URL after "] "
        if let Some(url) = desc.split("] ").nth(1) {
            if let Err(e) = Command::new("xdg-open").arg(url.trim()).spawn() {
                eprintln! ("[bookmarks-launcher] Failed to open URL: {}", e);
            }
        }
    }

    HandleResult::Close
}
