# Rinkle - Project Specification (V1.0)

## 1. Vision & Philosophy

### 1.1. Project Slogan
**"Ridiculously Fast Dotfiles Management"**

### 1.2. Core Experience: "Powerful and Fun"

`rinkle` is not just another utility; it's a power tool designed to be a joy to use.

*   **Powerful** means:
    *   **Speed:** Leveraging Rust to be faster than shell-script-based alternatives.
    *   **Control:** Giving users fine-grained control over their configurations via the `rinkle.toml` file.
    *   **Flexibility:** Handling simple use cases and complex multi-machine, multi-profile scenarios with equal elegance.

*   **Fun** means:
    *   **Helpful Feedback:** Error messages are clear, friendly, and guide the user toward a solution.
    *   **Satisfying UI:** A well-designed command-line interface, with satisfying progress bars and informative status displays.
    *   **Discovery:** An interactive mode that encourages exploration and makes managing dotfiles feel less like a chore and more like organizing a digital home.

## 2. Target Audience & User Personas

### 2.1. Persona 1: Alex, the Newcomer
*   **Profile:** A developer who has heard about "dotfiles" and wants to sync their `.zshrc` and editor configs between their two machines (desktop and laptop).
*   **Needs:** A simple, guided experience. They don't want to read pages of documentation to get started.
*   **Rinkle's Solution for Alex:** The `rinkle init` command provides a "one-shot" setup that clones their repository, asks simple questions, generates the config file, and links their files.

### 2.2. Persona 2: Sam, the Power User
*   **Profile:** An experienced developer with a sophisticated dotfiles repository supporting Linux at home and macOS at work. They have different Git configurations and tools for each environment.
*   **Needs:** Profiles, OS-specific rules, versioning for their Neovim config (testing `nightly` vs. `stable`), and robust conflict management.
*   **Rinkle's Solution for Sam:** The `rinkle.toml` file provides the expressive power to define profiles, OS-specific packages, and versioning strategies. The `rinkle use-profile` and `rinkle vsc` commands give them precise control over their environment.

## 3. Feature Scope

### 3.1. In-Scope Features
*   **Package Management:** The core concept of defining manageable configuration units ("packages").
*   **Linking:** The primary mechanism for synchronizing files via symlinks.
*   **Profiles:** High-level collections of packages for different scenarios (e.g., `work`, `home`).
*   **Versioning:** Support for managing different versions of a package via naming conventions (e.g., `nvim@stable`).
*   **OS-Specificity:** The ability to tag packages for specific operating systems (`linux`, `macos`). Windows is currently out of scope.
*   **Conflict Resolution:** Safe handling of pre-existing files in target directories.
*   **Live Monitoring:** A background service to auto-sync changes.
*   **Guided Setup:** A `rinkle init` command for a seamless first-time experience.
*   **Interactive Mode:** A CLI/TUI for managing `rinkle`.
*   **VSC Caching:** Cache discovered version directories to speed up resolution.
*   **File Locking:** Use a lock file when reading/writing state to avoid concurrent corruption.

### 3.2. Out-of-Scope Features
*   **Templating:** `rinkle` will not generate file content dynamically. It only links existing files.
*   **Secrets Management:** `rinkle` will not handle encryption or management of sensitive data within dotfiles.
*   **Windows Support:** Deferred. Not supported in the initial versions.
*   **Fallback Strategies:** No automatic fallbacks (copy/hardlink) when symlink is unavailable; users should resolve issues manually.
*   **One-Click Clean:** No single command to delete backups or auto-clean artifacts.
*   **Interop/Import From Other Tools:** Not considered for now (e.g., GNU Stow import).

## 4. Configuration (`rinkle.toml`)

The `rinkle.toml` file is the declarative "brain" of the application.

### 4.1. High-Level Structure
```toml
# 1. Global defaults and settings
[global]
# ...

# 2. Versioning strategy definition
[vsc]
# ...

# 3. Profile definitions
[profiles]
# ...

# 4. Individual package definitions
[packages]
# ...
```

### 4.2. Detailed Sections

#### `[global]`
Defines default behaviors.
```toml
[global]
# The root directory of your dotfiles repository.
source_dir = "~/dotfiles"

# The base directory where symlinks will be created.
# A package named 'nvim' will be linked to `~/.config/nvim` by default.
target_dir = "~/.config"

# Strategy for when a target file already exists and is NOT a symlink.
# - "skip": Do nothing and report a warning.
# - "overwrite": Replace the target file with the symlink. (Potentially Destructive)
# - "backup": Rename the existing file (e.g., `file.bak`) before creating the symlink. (Safe)
# - "prompt": Ask the user what to do.
conflict_strategy = "backup"

# Glob patterns for files/directories to ignore globally.
ignore = [".git/", "**/.DS_Store", "README.md"]
```

#### `[vsc]`
Defines the strategy for versioned packages.
```toml
[vsc]
# A regex with a named capture group `version` to extract the version from a directory name.
template = ".*@(?P<version>[a-zA-Z0-9_.-]+)$"
# The default version to use if a package is versioned but no version is specified.
default_version = "stable"
```
*Example File Structure:*
```
~/dotfiles/
├── nvim@stable/
│   └── init.lua
└── nvim@nightly/
    └── init.lua
```

#### `[profiles]`
Groups packages using tags for easy switching.
```toml
[profiles]
# The 'default' profile is active if no other profile is specified.
default = ["common", "cli-tools"]
work = ["common", "cli-tools", "work-tools"]
home = ["common", "games"]
```

#### `[packages]`
The heart of the configuration, where each package is defined.
```toml
[packages]

# A simple package using global settings.
# It belongs to the "common" group, so it's in the 'default' and 'work' profiles.
[packages.zsh]
tags = ["common"]

# A package overriding globals and specifying an OS.
[packages.kitty]
source = "kitty/" # Relative to global.source_dir
target = "~/.config/kitty"
os = ["linux"]
tags = ["common"]

# A package for a specific profile with its own conflict strategy.
[packages.git-work]
source = "git/gitconfig-work"
target = "~/.gitconfig"
conflict_strategy = "overwrite"
tags = ["work-tools"] # Only included in the 'work' profile.

# A versioned package.
[packages.nvim]
tags = ["common"]
# Overrides the global default version for this package specifically.
default_version = "nightly"
```

## 5. Command-Line Interface (CLI)

### 5.1. Configuration Precedence
`rinkle` resolves settings in the following order of priority (higher takes precedence):
1.  **Command-Line Flags** (e.g., `rinkle link --conflict-strategy=overwrite`)
2.  **State File** (e.g., pinned versions in `state.toml`)
3.  **`rinkle.toml`** (Package-specific settings > Global settings)
4.  **Internal Defaults**

### 5.2. Key Commands
*   `rinkle init [git_repo]`: Guides a new user through cloning their repo and generating a `rinkle.toml`.
*   `rinkle link [pkg...]`: Links packages. Can specify versions like `nvim@stable`.
*   `rinkle use-profile <name>`: Switches the active profile by updating `state.toml`.
*   `rinkle vsc <pkg> <ver>`: Pins the default version for a package by updating `state.toml`.
*   `rinkle status`: Provides a rich overview of package status.
*   `rinkle start`: Starts the live-monitoring daemon.

Notes:
- There is no `clean` command in the initial scope.

## 6. State Management (`state.toml`)

To maintain state between commands, `rinkle` will use a state file.
*   **Location:** `~/.config/rinkle/state.toml`
*   **Purpose:** To store user choices like the active profile or pinned package versions.
*   **Locking:** All reads/writes MUST acquire an exclusive file lock to avoid race conditions (e.g., flock/advisory lock).

*Example `state.toml`:*
```toml
# This file is managed by rinkle. Do not edit manually.
active_profile = "work"

[pinned_versions]
nvim = "nightly"
```

## 7. Future Roadmap

*   **Interactive TUI:** Evolve the `interactive` mode from a simple REPL into a full-fledged, visually appealing Terminal User Interface.
*   **Hooks:** Potentially add `pre-link` and `post-link` script execution hooks for advanced automation (e.g., installing dependencies).
*   **Performance Dashboard:** A command to show performance metrics, reinforcing the "fast" philosophy.
