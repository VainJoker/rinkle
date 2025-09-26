# Rinkle

[![Build](https://github.com/VainJoker/rinkle/actions/workflows/integration.yml/badge.svg)](https://github.com/VainJoker/rinkle/actions/workflows/integration.yml)
[![License: GPLv3](https://img.shields.io/badge/License-GPL-green.svg)](https://opensource.org/license/gpl-3-0)
[![Latest Version](https://img.shields.io/crates/v/rinkle.svg)](https://crates.io/crates/rinkle)
[![codecov](https://codecov.io/github/VainJoker/rinkle/graph/badge.svg?token=KF87R60IJ1)](https://codecov.io/github/VainJoker/rinkle)

Rinkle is a symlink farm manager for your dotfiles and other configurations. It helps you manage distinct sets of software and/or data located in separate directories and makes them appear to be installed in a single directory tree.

It is heavily inspired by tools like `stow` but with a focus on profiles and versioning, controlled by a simple TOML configuration file.

## Features

- **TOML Configuration**: Simple and expressive configuration.
- **Profiles**: Manage different sets of packages for different machines or environments (e.g., `work`, `personal`).
- **Versioning**: Switch between different versions of a package (e.g., `nvim-stable`, `nvim-nightly`).
- **Conflict Management**: Choose how to handle existing files (`skip`, `overwrite`, `backup`).
- **Status Checks**: Quickly see the state of your symlinks.
- **Interactive Mode**: A simple REPL for running commands.
- **File Monitoring**: (MVP) Automatically detect changes in your source directory.

## Installation

### From Crates.io

```bash
cargo install rinkle
```

### From Source

```bash
git clone https://github.com/vainjoker/rinkle.git
cd rinkle
cargo install --path .
```

## Getting Started

1.  **Initialize Rinkle**:
    Run the `init` command in your dotfiles repository. This will create a `config/rinkle.toml` for you.

    ```bash
    # In your dotfiles directory
    rinkle init
    ```
    This will guide you through an interactive setup.

2.  **Configure `rinkle.toml`**:
    Define your packages and profiles.

    ```toml
    # config/rinkle.toml
    [global]
    source_dir = "~/dotfiles"      # Your dotfiles location
    target_dir = "~/.config"       # Where to create symlinks
    conflict_strategy = "backup"   # or "skip", "overwrite"

    # Define profiles by mapping a name to a list of tags
    [profiles]
    default = ["common"]
    work = ["common", "work"]

    # Define packages
    [packages]
    # A simple package named 'nvim'
    [packages.nvim]
    tags = ["common"] # Belongs to the 'common' tag

    # A versioned package
    [packages.alacritty]
    tags = ["common"]
    # Rinkle will look for dirs matching 'alacritty-*'
    # and you can pin a version like 'alacritty@v0.12'

    [vsc]
    # Regex for versioned packages. Must contain a 'version' capture group.
    template = '^(?P<package>.+)-(?P<version>[vV]?\d+\.\d+\.\d+)$'
    default_version = "stable"
    ```

3.  **Link your files**:
    Use the `link` command. This will link all packages in your default profile.

    ```bash
    rinkle link
    ```

## Usage

- `rinkle init`: Interactively create a `rinkle.toml` config file.
- `rinkle list`: List all packages defined in the config.
- `rinkle status`: Check the status of packages for the active profile.
- `rinkle link [pkg1] [pkg2]...`: Link packages. If none are specified, links all packages from the active profile.
- `rinkle remove [pkg1] [pkg2]...`: Remove links.
- `rinkle use-profile <name>`: Set the active profile (e.g., `work`).
- `rinkle vsc <package> <version>`: Pin a specific version for a package.
- `rinkle start`: Start monitoring the source directory for changes (runs in foreground).
- `rinkle interactive`: Enter the interactive REPL mode.

### Global Flags

- `--dry-run`: Simulate the command without making any changes to the filesystem.
- `--config <path>`: Use a custom config file path.
- `--profile <name>`: Temporarily override the active profile for a single command.

## Platform Support

- Linux and macOS only for now. Windows is not supported yet.

## License

This project is distributed under the terms of GPLv3.

See [LICENSE](LICENSE) for details.

Copyright 2024 Jasper Zhang
