use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::Project;
use crate::error::F0Error;
use crate::process;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Slot {
    pub name: String,
    pub path: PathBuf,
    pub status: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub lease_holder: String,
}

/// Leases a slot with a new branch cut from the project's base and returns its path.
pub fn lease(project: &Project, branch: &str, holder: &str) -> Result<PathBuf, F0Error> {
    let mut args: Vec<String> = vec![
        "get".into(),
        "--lease".into(),
        "--lease-holder".into(),
        holder.into(),
        "--branch".into(),
        branch.into(),
    ];
    if let Some(base) = &project.base {
        args.extend(["--base".into(), base.clone()]);
    }
    if let Some(include) = &project.include_file {
        args.extend([
            "--include-file".into(),
            include.to_string_lossy().into_owned(),
        ]);
    }

    let path = process::output("treehouse", &args, Some(&project.repo))?;
    Ok(PathBuf::from(path))
}

pub fn status(repo: &Path) -> Result<Vec<Slot>, F0Error> {
    let json = process::output("treehouse", ["status", "--json"], Some(repo))?;
    parse_status(&json)
}

/// Returns the slot to the pool. Runs attached to the terminal because treehouse
/// asks for confirmation before discarding uncommitted changes.
/// With `force` it discards them without asking.
pub fn return_slot(path: &Path, force: bool) -> Result<(), F0Error> {
    let mut args = vec!["return".as_ref(), path.as_os_str()];
    if force {
        args.push("--force".as_ref());
    }
    process::interactive("treehouse", args, None)
}

fn parse_status(json: &str) -> Result<Vec<Slot>, F0Error> {
    serde_json::from_str(json).map_err(|source| F0Error::TreehouseStatus { source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_json() {
        let json = r#"[
            {"name":"1","path":"/p/1/hubi","status":"leased","branch":"feature/HB-1","flavor":"git",
             "lease_id":"x","lease_holder":"f0:hubi/HB-1","leased_at":null,"processes":[]},
            {"name":"2","path":"/p/2/hubi","status":"available","detached":true,"flavor":"git"}
        ]"#;
        let slots = parse_status(json).unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].branch, "feature/HB-1");
        assert_eq!(slots[0].lease_holder, "f0:hubi/HB-1");
        assert_eq!(slots[1].branch, "");
        assert_eq!(slots[1].path, PathBuf::from("/p/2/hubi"));
    }
}
