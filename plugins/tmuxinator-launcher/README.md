# tmuxinator anyrun launcher plugin

An [**anyrun**](https://github.com/anyrun-org/anyrun) plugin for discovering, creating, and launching **tmuxinator projects** with project detection and session actions.

## Features

- **Project scanning**
  - Scan directories with configurable depth
  - Supports multiple root directories

- **tmuxinator project discovery**
  - Optional custom global tmuxinator projects directory
  - Falls back to tmuxinator’s default locations if not specified
  - Uses filename as project name when applicable

- **Expand env variables and `~`**
  - Environment variables such as `$HOME` get expanded to `/home/USER/`
  - Tilde (`~`) gets expanded to `/home/USER/`

- **Project identification**
  - Searches for `.tmuxinator.yml` in a project root (definded with depth config)
  - Reads `project_name:` from config when in a local config

- **Automatic project creation**
  - Creates and runs a basic tmuxinator config file if one does not exist
  - Projects that need to be created are marked distinctly in the UI

- **Project action indicators**
  - Projects are tagged in the entry description field with:
    - `[attach]` – session exists, can attach
    - `[start]` – config exists, session not running
    - `[create]` – no config, can be generated

## Configuration

Example configuration:

```rust
Config(
  // Trigger prefix in anyrun
  prefix: ":t",

  // Optional: set the terminal to run in (otherwise defaults are used and the first match opens)
  terminal: "kitty",

  // Optional: custom global tmuxinator projects dir (otherwise defaults are used)
  // tmuxinator_dir: "~/tmuxinator-projects",

  // Directories to scan (path with ~ or $VARS is expanded; depth 0 = just the dir)
  directories: [
    (
      path: "$PROJECTS",
      depth: 1,
    ),
    (
      path: "~/.nix-dotfiles",
      depth: 0,
    ),
  ],
)
```

### Configuration Options

- `prefix`
  Command prefix used to trigger the launcher in anyrun.

- `tmuxinator_dir` *(optional)*
  Custom global directory for tmuxinator projects.
  If unset, default tmuxinator locations are used.

- `Directories`
  - `path`: Directory to scan
  - `depth`: Recursion depth (`0` = the directory itself, `1` = the directory's children)
