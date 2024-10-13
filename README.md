# rinkle

[![Build](https://github.com/VainJoker/rinkle/actions/workflows/integration.yml/badge.svg)](https://github.com/VainJoker/rinkle/actions/workflows/integration.yml) 
[![License: GPLv3](https://img.shields.io/badge/License-GPL-green.svg)](https://opensource.org/license/gpl-3-0) 
[![Latest Version](https://img.shields.io/crates/v/rinkle.svg)](https://crates.io/crates/rinkle) 
[![codecov](https://codecov.io/github/VainJoker/rinkle/graph/badge.svg?token=KF87R60IJ1)](https://codecov.io/github/VainJoker/rinkle)

Rinkle is a package management tool that operates based on the configuration in `config/rinkle.toml`.

## Usage

- `rk [package1] [package2] ...`: Process specified packages
- `rk list`: List all packages in config
- `rk remove [package1] [package2] ...`: Remove linked packages
- `rk remove --all`: Remove all linked packages
- `rk clean`: Clean all broken links
- `rk status`: Check status of all packages
- `rk vsc [package] [version]`: Select version for package
- `rk start`: Start monitoring packages (auto-link changes)
- `rk stop`: Stop monitoring packages
- `rk interactive`: Enter interactive mode
- `rk --dry-run`: Dry run all packages
- `rk -vvv`: Verbose mode
- `rk --config <path>`: Use custom config file
- `rk --help`: Show help
- `rk --version`: Show version

## License

This project is distributed under the terms of GPLv3.

See [LICENSE](LICENSE) for details.

Copyright 2024 Jasper Zhang