// linker_impl.rs
// Actual implementation moved here for clarity and to allow for public re-exports.

use std::path::{Path, PathBuf};

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum LinkStatusKind {
    Ok,
    BrokenSymlink,
    NotSymlink,
    Missing,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkStatus {
    pub package: String,
    pub target: PathBuf,
    pub kind: LinkStatusKind,
}

pub fn status_package(
    pkg_name: &str,
    pkg: &crate::config::entity::Package,
    cfg: &crate::config::entity::Config,
    state: &crate::state::State,
) -> Result<LinkStatus, Error> {
    let source_root = cfg.global.source_dir.as_deref().unwrap_or(".");
    let target_root = cfg.global.target_dir.as_deref().unwrap_or(".");
    let (source_dir, target_dir) = resolve_paths(pkg_name, pkg, source_root, target_root, cfg, state)?;

    // Determine kind
    let kind = if !target_dir.exists() {
        LinkStatusKind::Missing
    } else if target_dir.is_symlink() {
        match target_dir.read_link() {
            Ok(dest) => {
                if dest == source_dir { LinkStatusKind::Ok } else { LinkStatusKind::BrokenSymlink }
            }
            Err(_) => LinkStatusKind::BrokenSymlink,
        }
    } else {
        LinkStatusKind::NotSymlink
    };

    Ok(LinkStatus { package: pkg_name.to_string(), target: target_dir, kind })
}

pub fn link_package(
    name: &str,
    pkg: &crate::config::entity::Package,
    cfg: &crate::config::entity::Config,
    state: &crate::state::State,
    _ver: Option<&str>,
    dry_run: bool,
) -> Result<(), Error> {
    let source_root = cfg.global.source_dir.as_deref().unwrap_or(".");
    let target_root = cfg.global.target_dir.as_deref().unwrap_or(".");
    let (source_dir, target_dir) = resolve_paths(name, pkg, source_root, target_root, cfg, state)?;

    if !source_dir.exists() {
        return Err(Error::InvalidConfig(format!("source missing: {}", source_dir.display())));
    }

    std::fs::create_dir_all(target_dir.parent().unwrap())?;

    if target_dir.exists() {
        if target_dir.is_symlink() {
            // Already a symlink
            if target_dir.read_link()? == source_dir { return Ok(()); }
            // points elsewhere -> remove
            if !dry_run { std::fs::remove_file(&target_dir)?; }
        } else {
            // conflict
            handle_conflict(&target_dir, cfg.global.conflict_strategy, dry_run)?;
        }
    }

    if !dry_run {
        std::os::unix::fs::symlink(&source_dir, &target_dir)?;
    }
    Ok(())
}

pub fn remove_package(
    name: &str,
    pkg: &crate::config::entity::Package,
    cfg: &crate::config::entity::Config,
    state: &crate::state::State,
    _ver: Option<&str>,
    dry_run: bool,
) -> Result<(), Error> {
    let source_root = cfg.global.source_dir.as_deref().unwrap_or(".");
    let target_root = cfg.global.target_dir.as_deref().unwrap_or(".");
    let (_source_dir, target_dir) = resolve_paths(name, pkg, source_root, target_root, cfg, state)?;
    if target_dir.exists() && target_dir.is_symlink() {
        if !dry_run { std::fs::remove_file(target_dir)?; }
    }
    Ok(())
}

/// Expand a path that may contain `~` or environment variables.
///
/// Currently supports:
/// - `~` / `~/sub/path` -> user's home directory
/// - `$VAR` environment variable prefixes
pub fn expand_path<P: AsRef<str>>(p: P) -> PathBuf {
    let raw = p.as_ref();
    // Use shellexpand for simplicity (already a dependency via Cargo.toml)
    let expanded = shellexpand::full(raw).unwrap_or_else(|_| std::borrow::Cow::Borrowed(raw));
    Path::new(expanded.as_ref()).to_path_buf()
}

fn resolve_paths(
    name: &str,
    pkg: &crate::config::entity::Package,
    source_root: &str,
    target_root: &str,
    cfg: &crate::config::entity::Config,
    state: &crate::state::State,
) -> Result<(PathBuf, PathBuf), Error> {
    let version = pick_version(name, pkg, cfg, state).ok();
    let base_source = pkg.source.as_deref().unwrap_or(name);
    let mut source_dir = expand_path(format!("{}/{}", source_root, base_source));
    if let Some(ver) = version {
        // versioned directory convention: name@version
        let ver_path = expand_path(format!("{}/{}@{}", source_root, name, ver));
        if ver_path.exists() { source_dir = ver_path; }
    }
    let target_name = pkg.target.as_deref().unwrap_or_else(|| name);
    let target_dir = expand_path(format!("{}/{}", target_root, target_name));
    Ok((source_dir, target_dir))
}

fn pick_version(
    name: &str,
    pkg: &crate::config::entity::Package,
    cfg: &crate::config::entity::Config,
    state: &crate::state::State,
) -> Result<String, Error> {
    if let Some(pin) = state.pinned_versions.get(name) { return Ok(pin.clone()); }
    if let Some(p) = pkg.default_version.clone() { return Ok(p); }
    if let Some(p) = cfg.vsc.default_version.clone() { return Ok(p); }
    Err(Error::Version(format!("no version for {name}")))
}

fn handle_conflict(path: &Path, strat: crate::config::entity::ConflictStrategy, dry_run: bool) -> Result<(), Error> {
    use crate::config::entity::ConflictStrategy::*;
    match strat {
        Skip => { /* do nothing */ },
        Overwrite => { if !dry_run { if path.is_dir() { std::fs::remove_dir_all(path)?; } else { std::fs::remove_file(path)?; } } },
        Backup => {
            if !dry_run {
                let mut backup = path.to_path_buf();
                backup.set_extension("bak");
                std::fs::rename(path, backup)?;
            }
        },
        Prompt => {
            // simple prompt fallback
            if atty::is(atty::Stream::Stdin) {
                eprint!("conflict at {} overwrite? [y/N] ", path.display());
                use std::io::{Read, Write};
                std::io::stdout().flush().ok();
                let mut buf = [0u8; 1];
                if std::io::stdin().read(&mut buf).ok().filter(|&n| n>0).map(|_| buf[0] == b'y' || buf[0] == b'Y').unwrap_or(false) {
                    if !dry_run { if path.is_dir() { std::fs::remove_dir_all(path)?; } else { std::fs::remove_file(path)?; } }
                } else {
                    return Err(Error::Conflict(format!("user skipped {}", path.display())));
                }
            } else {
                return Err(Error::Conflict("prompt strategy not usable in non-interactive mode".into()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::entity::{Config, Global, Package, ConflictStrategy, Vsc}, state::State};
    use tempfile::tempdir;

    fn base_cfg(source: &str, target: &str) -> Config {
        Config {
            global: Global {
                source_dir: Some(source.to_string()),
                target_dir: Some(target.to_string()),
                conflict_strategy: ConflictStrategy::Backup,
                ignore: vec![],
            },
            vsc: Vsc { template: None, default_version: None },
            profiles: Default::default(),
            packages: Default::default(),
        }
    }

    #[test]
    fn status_missing_then_ok() {
        let src = tempdir().unwrap();
        let tgt = tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("nvim")).unwrap();
        let mut cfg = base_cfg(src.path().to_str().unwrap(), tgt.path().to_str().unwrap());
        cfg.packages.insert("nvim".into(), Package { ..Default::default() });
        let state = State::default();

        let st1 = status_package("nvim", cfg.packages.get("nvim").unwrap(), &cfg, &state).unwrap();
        assert!(matches!(st1.kind, LinkStatusKind::Missing));

        link_package("nvim", cfg.packages.get("nvim").unwrap(), &cfg, &state, None, false).unwrap();
        let st2 = status_package("nvim", cfg.packages.get("nvim").unwrap(), &cfg, &state).unwrap();
        assert!(matches!(st2.kind, LinkStatusKind::Ok));
    }

    #[test]
    fn status_not_symlink_and_broken() {
        let src = tempdir().unwrap();
        let tgt = tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("pkg")).unwrap();
        let mut cfg = base_cfg(src.path().to_str().unwrap(), tgt.path().to_str().unwrap());
        cfg.packages.insert("pkg".into(), Package { ..Default::default() });
        let state = State::default();

        // create regular file at target path
        let target_path = tgt.path().join("pkg");
        std::fs::write(&target_path, b"data").unwrap();
        let st_not = status_package("pkg", cfg.packages.get("pkg").unwrap(), &cfg, &state).unwrap();
        assert!(matches!(st_not.kind, LinkStatusKind::NotSymlink));

        // Replace with symlink pointing elsewhere to simulate broken
        std::fs::remove_file(&target_path).unwrap();
        let other_dir = src.path().join("other");
        std::fs::create_dir_all(&other_dir).unwrap();
        std::os::unix::fs::symlink(&other_dir, &target_path).unwrap();
        // real source dir for pkg
        let _ = &src.path().join("pkg");
        let st_broken = status_package("pkg", cfg.packages.get("pkg").unwrap(), &cfg, &state).unwrap();
        assert!(matches!(st_broken.kind, LinkStatusKind::BrokenSymlink));
    }
}
