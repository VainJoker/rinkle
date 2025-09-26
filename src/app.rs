use std::path::PathBuf;

use tracing::{
	error,
	info,
};

use crate::{
	cli::{
		Cli,
		Commands,
	},
	config,
	linker,
	monitor,
	repl,
	setup,
	state,
};

/// The main application structure.
///
/// This struct holds the application's configuration and state, acting as a
/// controller that executes commands based on user input.
pub struct App {
	/// The path to the `rinkle.toml` configuration file.
	config_path:      PathBuf,
	/// Flag indicating whether to perform a dry run.
	dry_run:          bool,
	/// An optional profile name that overrides the active profile for a single
	/// command.
	profile_override: Option<String>,
}

impl App {
	/// Creates a new `App` instance from the command-line arguments.
	///
	/// It resolves the configuration path and sets up the initial state based
	/// on the parsed `Cli` struct.
	pub fn new(cli: &Cli) -> Self {
		let config_path = cli
			.config
			.clone()
			.unwrap_or_else(|| config::loader::default_config_path());

		Self {
			config_path,
			dry_run: cli.dry_run,
			profile_override: cli.profile.clone(),
		}
	}

	/// Runs the command specified in the command-line arguments.
	///
	/// This is the main entry point for executing a command. It dispatches to
	/// the appropriate handler function based on the `Commands` enum variant.
	pub fn run(
		&self,
		command: &Commands,
	) -> Result<(), Box<dyn std::error::Error>> {
		match command {
			Commands::List => self.handle_list(),
			Commands::Status { json } => self.handle_status(*json),
			Commands::Link { packages } => self.handle_link(packages),
			Commands::Remove { packages } => self.handle_remove(packages),
			Commands::UseProfile { name } => self.handle_use_profile(name),
			Commands::Vsc { package, version } => {
				self.handle_vsc(package, version)
			}
			Commands::Init { repo, dest } => self.handle_init(repo, dest),
			Commands::Interactive => self.handle_interactive(),
			Commands::Start => self.handle_start(),
			Commands::Stop => self.handle_stop(),
		}
	}

	fn load_config_and_state(
		&self,
	) -> Result<
		(config::entity::Config, state::State),
		Box<dyn std::error::Error>,
	> {
		let cfg = config::loader::load_config(&self.config_path)?;
		let state =
			state::load_state(&state::default_state_path()).unwrap_or_default();
		Ok((cfg, state))
	}

	fn handle_list(&self) -> Result<(), Box<dyn std::error::Error>> {
		let (cfg, _) = self.load_config_and_state()?;
		if cfg.packages.is_empty() {
			println!("No packages defined in {}", self.config_path.display());
		} else {
			for name in cfg.packages.keys() {
				println!("{}", name);
			}
		}
		Ok(())
	}

	fn handle_status(
		&self,
		json: bool,
	) -> Result<(), Box<dyn std::error::Error>> {
		let (cfg, state) = self.load_config_and_state()?;
		info!("Loaded {} packages", cfg.packages.len());
		let filtered = select_packages(
			&cfg,
			self.profile_override.as_deref(),
			Some(&state),
		);

		if json {
			self.output_status_json(&filtered, &cfg, &state)?;
		} else {
			self.output_status_text(&filtered, &cfg, &state)?;
		}
		Ok(())
	}

	fn output_status_json(
		&self,
		packages: &[(&str, &config::entity::Package)],
		cfg: &config::entity::Config,
		state: &state::State,
	) -> Result<(), Box<dyn std::error::Error>> {
		#[derive(serde::Serialize)]
		struct Item<'a> {
			name:   &'a str,
			target: String,
			status: &'a str,
		}

		let mut items = Vec::new();
		for (name, pkg) in packages {
		match crate::linker::status_package(name, pkg, cfg, state) {
				Ok(st) => {
					let status = match st.kind {
						linker::LinkStatusKind::Ok => "ok",
						linker::LinkStatusKind::BrokenSymlink => "broken",
						linker::LinkStatusKind::NotSymlink => "not-symlink",
						linker::LinkStatusKind::Missing => "missing",
					};
					items.push(Item {
						name,
						target: st.target.display().to_string(),
						status,
					});
				}
				Err(_) => {
					items.push(Item {
						name,
						target: String::new(),
						status: "error",
					});
				}
			}
		}
		println!("{}", serde_json::to_string_pretty(&items)?);
		Ok(())
	}

	fn output_status_text(
		&self,
		packages: &[(&str, &config::entity::Package)],
		cfg: &config::entity::Config,
		state: &state::State,
	) -> Result<(), Box<dyn std::error::Error>> {
		use colored::Colorize;
		for (name, pkg) in packages {
		match crate::linker::status_package(name, pkg, cfg, state) {
				Ok(st) => {
					let status = match st.kind {
						linker::LinkStatusKind::Ok => "ok".green(),
						linker::LinkStatusKind::BrokenSymlink => "broken".red(),
						linker::LinkStatusKind::NotSymlink => "not-symlink".yellow(),
						linker::LinkStatusKind::Missing => "missing".dimmed(),
					};
					println!("{}: {} -> {}", name.bold(), status, st.target.display());
				}
				Err(e) => {
					eprintln!("{}: {}", name.bold(), format!("error: {e}").red());
				}
			}
		}
		Ok(())
	}

	fn handle_link(
		&self,
		packages: &[String],
	) -> Result<(), Box<dyn std::error::Error>> {
		self.process_packages(packages, "link", |name, pkg, cfg, state, ver| {
			linker::link_package(name, pkg, cfg, state, ver, self.dry_run)
		})
	}

	fn handle_remove(
		&self,
		packages: &[String],
	) -> Result<(), Box<dyn std::error::Error>> {
		self.process_packages(packages, "remove", |name, pkg, cfg, state, ver| {
			linker::remove_package(name, pkg, cfg, state, ver, self.dry_run)
		})
	}

	fn process_packages<F>(
        &self,
        packages: &[String],
        action_name: &str,
        action: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(
            &str,
            &config::entity::Package,
            &config::entity::Config,
            &state::State,
            Option<&str>,
	) -> Result<(), crate::error::Error>,
    {
        let (cfg, state) = self.load_config_and_state()?;
        let selected = self.resolve_package_list(packages, &cfg, &state);

        let bar = if selected.len() > 1 {
            indicatif::ProgressBar::new(selected.len() as u64)
        } else {
            indicatif::ProgressBar::hidden()
        };
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        for raw in selected {
            let (name, ver_override) = parse_pkg_and_version(&raw);
            bar.set_message(name.clone());
            match cfg.packages.get(&name) {
                Some(pkg) => {
                    if let Err(e) = action(
                        &name,
                        pkg,
                        &cfg,
                        &state,
                        ver_override.as_deref(),
                    ) {
                        error!("{action_name} {name} failed: {e}");
                    }
                }
                None => eprintln!("unknown package: {name}"),
            }
            bar.inc(1);
        }
        bar.finish_with_message("Done");
        Ok(())
    }

	fn resolve_package_list(
		&self,
		packages: &[String],
		cfg: &config::entity::Config,
		state: &state::State,
	) -> Vec<String> {
		if packages.is_empty() {
			select_packages(cfg, self.profile_override.as_deref(), Some(state))
				.into_iter()
				.map(|(n, _)| n.to_string())
				.collect()
		} else {
			packages.to_vec()
		}
	}

	fn handle_use_profile(
		&self,
		name: &str,
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut st =
			state::load_state(&state::default_state_path()).unwrap_or_default();
		st.active_profile = Some(name.to_string());
		state::save_state(&state::default_state_path(), &st)?;
		println!("active profile set to {name}");
		Ok(())
	}

	fn handle_vsc(
		&self,
		package: &str,
		version: &str,
	) -> Result<(), Box<dyn std::error::Error>> {
		let mut st =
			state::load_state(&state::default_state_path()).unwrap_or_default();
		st.pinned_versions
			.insert(package.to_string(), version.to_string());
		state::save_state(&state::default_state_path(), &st)?;
		println!("pinned {package} = {version}");
		Ok(())
	}

	fn handle_init(
		&self,
		repo: &Option<String>,
		dest: &Option<PathBuf>,
	) -> Result<(), Box<dyn std::error::Error>> {
		setup::init(repo.clone(), dest.clone())?;
		Ok(())
	}

	fn handle_interactive(&self) -> Result<(), Box<dyn std::error::Error>> {
		repl::run()?;
		Ok(())
	}

	fn handle_start(&self) -> Result<(), Box<dyn std::error::Error>> {
		let (cfg, _) = self.load_config_and_state()?;
		monitor::run_foreground(&cfg)?;
		Ok(())
	}

	fn handle_stop(&self) -> Result<(), Box<dyn std::error::Error>> {
		monitor::stop()?;
		Ok(())
	}
}

fn select_packages<'a>(
	cfg: &'a config::entity::Config,
	profile: Option<&str>,
	state: Option<&state::State>,
) -> Vec<(&'a str, &'a config::entity::Package)> {
	let effective_profile = profile
		.or_else(|| state.and_then(|s| s.active_profile.as_deref()))
		.unwrap_or("default");
	let active_tags: Option<&Vec<String>> = cfg.profiles.get(effective_profile);

	let os = current_os();
	if let Some(tags) = active_tags {
		let set: std::collections::HashSet<&str> =
			tags.iter().map(|s| s.as_str()).collect();
		cfg.packages
			.iter()
			.filter(|(_name, pkg)| {
				pkg.tags.iter().any(|t| set.contains(t.as_str()))
			})
			.filter(|(_name, pkg)| {
				pkg.os.is_empty() || pkg.os.iter().any(|v| v == os)
			})
			.map(|(n, p)| (n.as_str(), p))
			.collect()
	} else {
		cfg.packages
			.iter()
			.filter(|(_name, pkg)| {
				pkg.os.is_empty() || pkg.os.iter().any(|v| v == os)
			})
			.map(|(n, p)| (n.as_str(), p))
			.collect()
	}
}

fn parse_pkg_and_version(input: &str) -> (String, Option<String>) {
	if let Some((name, ver)) = input.split_once('@') {
		(name.to_string(), Some(ver.to_string()))
	} else {
		(input.to_string(), None)
	}
}

fn current_os() -> &'static str {
	if cfg!(target_os = "linux") {
		"linux"
	} else if cfg!(target_os = "macos") {
		"macos"
	} else {
		"other"
	}
}
