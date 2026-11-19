//! The executable of f0.app, a copy of Ghostty.app that install.sh renames. macOS runs it
//! when the app opens; it replaces itself with the Ghostty next to it, told to read only
//! f0's Ghostty config. Finder passes no arguments, and Ghostty has no other way to pick
//! a config file, so the flags have to come from here. A shell script cannot take this
//! place: launchd refuses to start an app whose executable is a script (POSIX 162).
//!
//! `XDG_CONFIG_HOME` is left alone on purpose: the panes inherit Ghostty's environment,
//! and f0 reads its own config from there.
//!
//! An app opened from Finder gets launchd's minimal `PATH`, and the login shell f0.conf
//! starts does not read `~/.zshrc`, where many installers add `~/.local/bin`. So before
//! starting Ghostty, this asks the user's shell for its `PATH` once, the way a terminal
//! would see it, and adds a few well-known folders after it.

use std::ffi::OsString;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

/// Marks the `PATH` in the shell's output, so whatever its rc files print is skipped.
const MARK: &str = "__F0_PATH__";
/// How long the user's shell may take to start before f0.app gives up on it.
const SHELL_TIMEOUT: Duration = Duration::from_secs(3);
/// launchd's `PATH` for apps, used when there is none at all.
const SYSTEM_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

fn main() -> ExitCode {
    let Some(ghostty) = ghostty_path() else {
        eprintln!("f0.app: cannot find the Ghostty executable next to this one");
        return ExitCode::FAILURE;
    };
    let Some(config) = config_file(
        std::env::var_os("F0_CONFIG_DIR"),
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    ) else {
        eprintln!("f0.app: HOME is not set, so f0's Ghostty config cannot be found");
        return ExitCode::FAILURE;
    };
    let home = std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from);
    let path = full_path(
        shell_path(),
        std::env::var_os("PATH"),
        &fallback_dirs(home.as_deref()),
        |dir| dir.is_dir(),
    );
    let error = Command::new(&ghostty)
        .env("PATH", path)
        .args(ghostty_args(&config, std::env::args_os().skip(1)))
        .exec();
    eprintln!("f0.app: could not start {}: {error}", ghostty.display());
    ExitCode::FAILURE
}

/// `Contents/MacOS/ghostty`, beside this executable.
fn ghostty_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
    Some(exe.parent()?.join("ghostty"))
}

/// `ghostty.conf` in f0's config folder, found the way the f0 CLI finds it:
/// `F0_CONFIG_DIR`, else `$XDG_CONFIG_HOME/f0`, else `~/.config/f0`. Empty variables
/// count as unset.
fn config_file(
    f0_dir: Option<OsString>,
    xdg: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    let set = |value: Option<OsString>| value.filter(|v| !v.is_empty()).map(PathBuf::from);
    let dir = match (set(f0_dir), set(xdg)) {
        (Some(dir), _) => dir,
        (None, Some(xdg)) => xdg.join("f0"),
        (None, None) => set(home)?.join(".config/f0"),
    };
    Some(dir.join("ghostty.conf"))
}

/// Only f0's config, then whatever the app was opened with. macOS may add a `-psn_`
/// process serial number, which Ghostty would reject.
fn ghostty_args(config: &Path, args: impl Iterator<Item = OsString>) -> Vec<OsString> {
    let mut flag = OsString::from("--config-file=");
    flag.push(config);
    let mut out = vec![OsString::from("--config-default-files=false"), flag];
    out.extend(args.filter(|arg| !arg.to_string_lossy().starts_with("-psn_")));
    out
}

/// The `PATH` an interactive login shell ends up with, which is what the user's terminal
/// sees. `None` when the shell fails, prints no marked `PATH`, or takes too long.
fn shell_path() -> Option<OsString> {
    let shell = std::env::var_os("SHELL")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| OsString::from("/bin/zsh"));
    // `/bin/sh` prints the exported PATH, so the command works in zsh, bash and fish alike.
    let script = format!("/bin/sh -c 'printf \"{MARK}%s{MARK}\" \"$PATH\"'");
    let mut child = Command::new(shell)
        .args(["-l", "-i", "-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        stdout.read_to_end(&mut out).map(|_| out)
    });
    let deadline = Instant::now() + SHELL_TIMEOUT;
    loop {
        if child.try_wait().ok()?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            // The whole group, so nothing the rc files started keeps the pipe open.
            let _ = Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", child.id())])
                .status();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = reader.join().ok()?.ok()?;
    marked_path(&String::from_utf8_lossy(&out)).map(OsString::from)
}

/// The text between the first two markers, if it is not empty.
fn marked_path(out: &str) -> Option<&str> {
    let (_, rest) = out.split_once(MARK)?;
    let (path, _) = rest.split_once(MARK)?;
    (!path.is_empty()).then_some(path)
}

/// Folders where agents and tools are often installed but not always on `PATH`.
fn fallback_dirs(home: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = home {
        dirs.push(home.join(".local/bin"));
    }
    dirs.push(PathBuf::from("/opt/homebrew/bin"));
    dirs.push(PathBuf::from("/usr/local/bin"));
    dirs
}

/// The shell's `PATH` (else the inherited one, else launchd's), then each fallback
/// folder that exists and is not on it yet.
fn full_path(
    shell: Option<OsString>,
    inherited: Option<OsString>,
    fallbacks: &[PathBuf],
    exists: impl Fn(&Path) -> bool,
) -> OsString {
    let base = shell
        .or(inherited)
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| OsString::from(SYSTEM_PATH));
    let mut dirs: Vec<PathBuf> = std::env::split_paths(&base).collect();
    for dir in fallbacks {
        if !dirs.contains(dir) && exists(dir) {
            dirs.push(dir.clone());
        }
    }
    std::env::join_paths(dirs).unwrap_or(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(value: &str) -> Option<OsString> {
        Some(OsString::from(value))
    }

    #[test]
    fn config_follows_the_f0_cli() {
        assert_eq!(
            config_file(None, None, os("/Users/me")),
            Some(PathBuf::from("/Users/me/.config/f0/ghostty.conf"))
        );
        assert_eq!(
            config_file(None, os("/x"), os("/Users/me")),
            Some(PathBuf::from("/x/f0/ghostty.conf"))
        );
        assert_eq!(
            config_file(os("/f0"), os("/x"), os("/Users/me")),
            Some(PathBuf::from("/f0/ghostty.conf"))
        );
        assert_eq!(
            config_file(os(""), os(""), os("/Users/me")),
            Some(PathBuf::from("/Users/me/.config/f0/ghostty.conf"))
        );
        assert_eq!(config_file(None, None, None), None);
    }

    #[test]
    fn args_load_only_the_f0_config_and_drop_the_serial_number() {
        let args = ["-psn_0_12345", "--title=x"]
            .map(OsString::from)
            .into_iter();
        assert_eq!(
            ghostty_args(Path::new("/c/ghostty.conf"), args),
            [
                "--config-default-files=false",
                "--config-file=/c/ghostty.conf",
                "--title=x"
            ]
            .map(OsString::from)
        );
    }

    #[test]
    fn marked_path_skips_what_the_rc_files_print() {
        assert_eq!(
            marked_path("welcome!\n__F0_PATH__/a:/b__F0_PATH__"),
            Some("/a:/b")
        );
        assert_eq!(marked_path("__F0_PATH____F0_PATH__"), None);
        assert_eq!(marked_path("no marker"), None);
        assert_eq!(marked_path("__F0_PATH__/a"), None);
    }

    #[test]
    fn fallbacks_start_with_the_home_bin() {
        assert_eq!(
            fallback_dirs(Some(Path::new("/Users/me"))),
            [
                "/Users/me/.local/bin",
                "/opt/homebrew/bin",
                "/usr/local/bin"
            ]
            .map(PathBuf::from)
        );
        assert_eq!(fallback_dirs(None).len(), 2);
    }

    #[test]
    fn path_prefers_the_shell_and_adds_missing_fallbacks() {
        let fallbacks = ["/home/.local/bin", "/opt/homebrew/bin", "/nope"].map(PathBuf::from);
        let exists = |dir: &Path| dir != Path::new("/nope");
        assert_eq!(
            full_path(
                os("/opt/homebrew/bin:/usr/bin"),
                os("/usr/bin"),
                &fallbacks,
                exists
            ),
            OsString::from("/opt/homebrew/bin:/usr/bin:/home/.local/bin")
        );
        assert_eq!(
            full_path(None, os("/usr/bin"), &fallbacks, exists),
            OsString::from("/usr/bin:/home/.local/bin:/opt/homebrew/bin")
        );
        assert_eq!(
            full_path(None, None, &[], exists),
            OsString::from(SYSTEM_PATH)
        );
    }
}
