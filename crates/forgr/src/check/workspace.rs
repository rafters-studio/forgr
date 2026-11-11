//! The workspace's members, from its manifest as `cargo metadata` reads it.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A workspace's root and the directory of each member, in name order.
pub(super) struct Workspace {
    pub root: PathBuf,
    pub members: Vec<PathBuf>,
}

/// The workspace that holds the crate at `manifest_dir`: a crate in no
/// workspace is the one member of its own.
pub(super) fn read(manifest_dir: &Path) -> Result<Workspace, String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest = manifest_dir.join("Cargo.toml");
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--no-deps",
            "--offline",
            "--format-version",
            "1",
        ])
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .map_err(|err| format!("could not run cargo metadata: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata could not read {}: {}",
            manifest.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("cargo metadata printed what is not JSON: {err}"))?;
    let root = metadata["workspace_root"]
        .as_str()
        .ok_or("cargo metadata named no workspace root")?;
    let ids = metadata["workspace_members"]
        .as_array()
        .ok_or("cargo metadata named no workspace members")?;
    let mut members = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        if !ids.contains(&package["id"]) {
            continue;
        }
        let path = package["manifest_path"]
            .as_str()
            .ok_or("cargo metadata named a member with no manifest")?;
        if let Some(dir) = Path::new(path).parent() {
            members.push(dir.to_path_buf());
        }
    }
    members.sort();
    Ok(Workspace {
        root: PathBuf::from(root),
        members,
    })
}
