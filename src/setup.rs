use std::path::{
	Path,
	PathBuf,
};

use dialoguer::{
	Confirm,
	Input,
	Select,
	theme::ColorfulTheme,
};

/// Handles the `init` command to set up a new rinkle project.
///
/// This function can optionally clone a git repository. It then checks for an
/// existing `rinkle.toml` and, if not found, launches an interactive prompt to
/// create one.
pub fn init(
	repo: Option<String>,
	dest: Option<PathBuf>,
) -> std::io::Result<()> {
	let root = dest.unwrap_or_else(|| std::env::current_dir().unwrap());
	if let Some(url) = repo {
		git_clone(&url, &root)?;
	}

	let cfg_dir = root.join("config");
	std::fs::create_dir_all(&cfg_dir)?;
	let cfg_path = cfg_dir.join("rinkle.toml");

	if !cfg_path.exists() {
		if Confirm::with_theme(&ColorfulTheme::default())
			.with_prompt("rinkle.toml not found. Create a new one?")
			   .interact().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
		{
			interactive_config(&cfg_path)?;
		}
	} else {
		println!("rinkle.toml already exists.");
	}

	println!("initialized rinkle at {}", root.display());
	Ok(())
}

fn git_clone(repo: &str, dest: &Path) -> std::io::Result<()> {
	std::fs::create_dir_all(dest)?;
	let status = std::process::Command::new("git")
		.args(["clone", repo, &dest.to_string_lossy()])
		.status()?;
	if !status.success() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"git clone failed",
		));
	}
	Ok(())
}

fn interactive_config(path: &Path) -> std::io::Result<()> {
	let theme = ColorfulTheme::default();
	let source_dir: String = Input::with_theme(&theme)
		.with_prompt("Source directory for your dotfiles?")
		.default("~/dotfiles".into())
		.interact_text().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

	let target_dir: String = Input::with_theme(&theme)
		.with_prompt("Target directory for symlinks?")
		.default("~/.config".into())
		.interact_text().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

	let strategies = &["skip", "overwrite", "backup"];
	let strategy_idx = Select::with_theme(&theme)
		.with_prompt("Default conflict strategy?")
		.items(strategies)
		.default(2) // backup
		.interact().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

	let content = format!(
		r#"[global]
source_dir = "{}"
target_dir = "{}"
conflict_strategy = "{}"

[packages]
# Example package
# [packages.nvim]
# tags = ["myprofile"]
"#,
		source_dir, target_dir, strategies[strategy_idx]
	);

	std::fs::write(path, content)?;
	println!("Created config at {}", path.display());
	Ok(())
}
