use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

fn bin() -> Command {
	Command::cargo_bin("rinkle").unwrap()
}

#[test]
fn list_and_status_json() {
	// Use repo's example config
	let mut cmd = bin();
	cmd.arg("list").arg("--config").arg("config/rinkle.toml");
	cmd.assert()
		.success()
		.stdout(predicate::str::contains("zsh"));

	let mut cmd = bin();
	cmd.args(["status", "--json"])
		.arg("--config")
		.arg("config/rinkle.toml");
	cmd.assert()
		.success()
		.stdout(predicate::str::contains("\"name\": \"zsh\""));
}

#[test]
fn dry_run_link_remove_ok() {
	// Should not fail even if directories don't exist
	let mut cmd = bin();
	cmd.args(["link", "--dry-run"])
		.arg("--config")
		.arg("config/rinkle.toml");
	cmd.assert().success();

	let mut cmd = bin();
	cmd.args(["remove", "zsh"])
		.arg("--dry-run")
		.arg("--config")
		.arg("config/rinkle.toml");
	cmd.assert().success();
}

#[test]
fn link_and_remove_real() {
	let dir = tempdir().unwrap();
	let src_root = dir.path().join("src");
	let dst_root = dir.path().join("dst");
	let cfg_root = dir.path().join("cfg");
	fs::create_dir_all(&src_root).unwrap();
	fs::create_dir_all(&dst_root).unwrap();
	fs::create_dir_all(&cfg_root).unwrap();

	let pkg_src = src_root.join("mypkg");
	fs::write(&pkg_src, "content").unwrap();

	let config_content = format!(
		r#"
[global]
source_dir = "{}"
target_dir = "{}"

[packages.mypkg]
"#,
		src_root.display(),
		dst_root.display()
	);
	let config_path = cfg_root.join("rinkle.toml");
	fs::write(&config_path, config_content).unwrap();

	// Run link command
	let mut cmd = bin();
	cmd.current_dir(dir.path())
		.args(["link", "mypkg", "--config"])
		.arg(&config_path);
	cmd.assert().success();

	// Check if symlink was created
	let dst_path = dst_root.join("mypkg");
	assert!(dst_path.exists());
	assert!(fs::symlink_metadata(&dst_path)
		.unwrap()
		.file_type()
		.is_symlink());
	assert_eq!(fs::read_link(&dst_path).unwrap(), pkg_src);

	// Run remove command
	let mut cmd = bin();
	cmd.current_dir(dir.path())
		.args(["remove", "mypkg", "--config"])
		.arg(&config_path);
	cmd.assert().success();

	// Check if symlink was removed
	assert!(!dst_path.exists());
}
