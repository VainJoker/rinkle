// cli for rinkle

// This will do all stuff in config/rinkle.toml
// rk [package1] [package2] ...

// List all packages in config/rinkle.toml
// rk list

// Remove linked packages in target_dir
// rk remove [package1] [package2] ...

// Remove all linked packages in target_dir
// rk remove --all

// Clean all broken links in target_dir
// rk clean

// Check status of all packages in config/rinkle.toml
// rk status

// Select version for package
// rk vsc [package] [version]

// Start monitoring all packages in config/rinkle.toml
// This will auto link packages that have been changed in 5s(you can change it
// in config/rinkle.toml) later rk start

// Stop monitoring all packages in config/rinkle.toml
// rk stop

// Interactive mode
// rk interactive

// Dry run all packages in config/rinkle.toml
// rk --dry-run

// Verbose mode
// rk -vvv

// Custom config file
// rk --config ./config/rinkle.toml

// Show help
// rk --help

// Version
// rk --version
