use std::{
	fs,
	process::Command,
};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

fn bin() -> Command {
	Command::cargo_bin("rinkle").unwrap()
}

// Helper to write a config file
fn write_cfg(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
	let cfg_path = dir.join("rinkle.toml");
	fs::write(&cfg_path, content).unwrap();
	cfg_path
}

#[test]
fn use_profile_and_filtering() {
	let tmp = tempdir().unwrap();
	let src = tmp.path().join("src");
	let dst = tmp.path().join("dst");
	fs::create_dir_all(&src).unwrap();
	fs::create_dir_all(&dst).unwrap();
	fs::create_dir_all(src.join("pkg1")).unwrap();
	fs::create_dir_all(src.join("pkg2")).unwrap();

	let cfg = format!(
		r#"[global]
source_dir = "{}"
target_dir = "{}"
[profiles]
work = ["w"]
play = ["p"]
[packages.pkg1]
tags = ["w"]
[packages.pkg2]
tags = ["p"]
"#,
		src.display(),
		dst.display()
	);
	let cfg_path = write_cfg(tmp.path(), &cfg);

	// status default -> none because no default profile tags
	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["status", "--config"])
		.arg(&cfg_path);
	cmd.assert().success();

	// set profile work
	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["use-profile", "work", "--config"])
		.arg(&cfg_path);
	cmd.assert()
		.stdout(predicate::str::contains("active profile set to work"));

	// link (should link only pkg1)
	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["link", "--config"])
		.arg(&cfg_path);
	cmd.assert().success();
	assert!(dst.join("pkg1").exists());
	assert!(!dst.join("pkg2").exists());
}

#[test]
fn vsc_version_selection_priority() {
	let tmp = tempdir().unwrap();
	let src = tmp.path().join("src");
	let dst = tmp.path().join("dst");
	// isolated state file
	let state_path = tmp.path().join("state.toml");
	std::env::set_var("RINKLE_STATE_PATH", &state_path);
	fs::create_dir_all(&src).unwrap();
	fs::create_dir_all(&dst).unwrap();
	fs::create_dir_all(src.join("tool@1")).unwrap();
	fs::create_dir_all(src.join("tool@2")).unwrap();
	// default_version at package level = 2, global vsc default = 1
	let cfg = format!(
		r#"[global]
source_dir = "{}"
target_dir = "{}"
[vsc]
default_version = "1"
[packages.tool]
default_version = "2"
"#,
		src.display(),
		dst.display()
	);
	let cfg_path = write_cfg(tmp.path(), &cfg);

	// Without pin, expect directory tool@2 picked
	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["link", "tool", "--config"])
		.arg(&cfg_path);
	cmd.assert().success();
	let link = dst.join("tool");
	assert!(link.exists());
	assert_eq!(fs::read_link(&link).unwrap(), src.join("tool@2"));

	// Pin to version 1
	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["vsc", "tool", "1", "--config"])
		.arg(&cfg_path);
	cmd.assert()
		.stdout(predicate::str::contains("pinned tool -> 1"));

	// Remove old link then re-link
	let _ = fs::remove_file(&link);
	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["link", "tool", "--config"])
		.arg(&cfg_path);
	cmd.assert().success();
	assert_eq!(fs::read_link(&link).unwrap(), src.join("tool@1"));
}

#[test]
fn conflict_strategy_backup() {
	let tmp = tempdir().unwrap();
	let src = tmp.path().join("src");
	let dst = tmp.path().join("dst");
	fs::create_dir_all(&src).unwrap();
	fs::create_dir_all(&dst).unwrap();
	fs::create_dir_all(src.join("pkg")).unwrap();

	let cfg = format!(
		r#"[global]
source_dir = "{}"
target_dir = "{}"
conflict_strategy = "backup"
[packages.pkg]
"#,
		src.display(),
		dst.display()
	);
	let cfg_path = write_cfg(tmp.path(), &cfg);

	// Create conflicting regular file
	let target_file = dst.join("pkg");
	fs::write(&target_file, b"data").unwrap();

	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["link", "pkg", "--config"])
		.arg(&cfg_path);
	cmd.assert().success();

	// original renamed to .bak and link created
	let mut bak = target_file.clone();
	bak.set_extension("bak");
	assert!(bak.exists());
	assert!(
		fs::symlink_metadata(&target_file)
			.unwrap()
			.file_type()
			.is_symlink()
	);
}

#[test]
fn dry_run_no_side_effects() {
	let tmp = tempdir().unwrap();
	let src = tmp.path().join("src");
	let dst = tmp.path().join("dst");
	fs::create_dir_all(&src).unwrap();
	fs::create_dir_all(&dst).unwrap();
	fs::create_dir_all(src.join("pkg")).unwrap();

	let cfg = format!(
		r#"[global]
source_dir = "{}"
target_dir = "{}"
[packages.pkg]
"#,
		src.display(),
		dst.display()
	);
	let cfg_path = write_cfg(tmp.path(), &cfg);

	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["link", "pkg", "--dry-run", "--config"])
		.arg(&cfg_path);
	cmd.assert().success();
	// No link created
	assert!(!dst.join("pkg").exists());
}

#[test]
fn status_json_structure() {
	let tmp = tempdir().unwrap();
	let src = tmp.path().join("src");
	let dst = tmp.path().join("dst");
	fs::create_dir_all(&src).unwrap();
	fs::create_dir_all(&dst).unwrap();
	fs::create_dir_all(src.join("a")).unwrap();

	let cfg = format!(
		r#"[global]
source_dir = "{}"
target_dir = "{}"
[packages.a]
"#,
		src.display(),
		dst.display()
	);
	let cfg_path = write_cfg(tmp.path(), &cfg);

	let mut cmd = bin();
	cmd.current_dir(tmp.path())
		.args(["status", "--json", "--config"])
		.arg(&cfg_path);
	cmd.assert().success().stdout(
		predicate::str::contains("\"name\": \"a\"")
			.or(predicate::str::contains("\"status\"")),
	);
}
