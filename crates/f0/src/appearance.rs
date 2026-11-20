//! The light or dark mode the workspace uses, and what keeps the bundled theme from
//! applying: a tmux without theme options, Ghostty themes missing from its themes
//! folder, and colors set in other Ghostty config files, which win over the bundled
//! `theme`.

use std::io;
use std::path::{Path, PathBuf};

use crate::process;

/// Light or dark, as picked in the launcher. `System` follows the macOS appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    System,
    Light,
    Dark,
}

impl Mode {
    /// The mode after this one, in the order the launcher cycles through them.
    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }

    /// The launcher's key hint for this mode.
    pub fn hint(self) -> &'static str {
        match self {
            Self::System => "theme: auto",
            Self::Light => "theme: light",
            Self::Dark => "theme: dark",
        }
    }

    /// Whether the workspace is dark in this mode.
    pub fn is_dark(self) -> bool {
        match self {
            Self::System => macos_dark(),
            Self::Light => false,
            Self::Dark => true,
        }
    }
}

/// Where the launcher keeps the mode: `light` or `dark`, or no file to follow the
/// system. A plain word, because config/tmux/appearance.sh and Neovim read it too.
pub fn mode_file(config_dir: &Path) -> PathBuf {
    config_dir.join("appearance")
}

/// The saved mode; anything but `light` or `dark` follows the system.
pub fn read_mode(file: &Path) -> Mode {
    match std::fs::read_to_string(file).as_deref().map(str::trim) {
        Ok("light") => Mode::Light,
        Ok("dark") => Mode::Dark,
        _ => Mode::System,
    }
}

pub fn write_mode(file: &Path, mode: Mode) -> io::Result<()> {
    let word = match mode {
        Mode::System => {
            return match std::fs::remove_file(file) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        Mode::Light => "light",
        Mode::Dark => "dark",
    };
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(file, format!("{word}\n"))
}

/// Whether macOS uses the dark appearance; anywhere else, light.
fn macos_dark() -> bool {
    process::output("defaults", ["read", "-g", "AppleInterfaceStyle"], None)
        .is_ok_and(|style| style == "Dark")
}

/// The first tmux with `theme`, `dark-theme-*` and `light-theme-*`.
pub const MIN_TMUX: (u32, u32) = (3, 8);

const GHOSTTY_COLOR_KEYS: [&str; 7] = [
    "background",
    "foreground",
    "palette",
    "cursor-color",
    "cursor-text",
    "selection-background",
    "selection-foreground",
];

/// The themes the bundled Ghostty config names, linked into Ghostty's themes folder.
pub const GHOSTTY_THEMES: [&str; 2] = ["f0-light", "f0-dark"];

/// The f0 themes missing from Ghostty's themes folder `dir`, including broken links.
pub fn missing_ghostty_themes(dir: &Path) -> Vec<&'static str> {
    GHOSTTY_THEMES
        .into_iter()
        .filter(|theme| !dir.join(theme).is_file())
        .collect()
}

/// Whether `link` is a symlink to `<repo>/<rel>` inside an f0 repo, as install.sh leaves it.
pub fn is_bundled(link: &Path, rel: &Path) -> bool {
    let Ok(target) = std::fs::read_link(link) else {
        return false;
    };
    let target = match link.parent() {
        Some(dir) if target.is_relative() => dir.join(target),
        _ => target,
    };
    let mut repo = target.as_path();
    for _ in rel.components() {
        match repo.parent() {
            Some(parent) => repo = parent,
            None => return false,
        }
    }
    repo.join(rel) == target && repo.join("crates/f0").is_dir()
}

/// Parses `tmux -V`: `tmux 3.8`, `tmux 3.5a` or `tmux next-3.8`.
pub fn tmux_version(out: &str) -> Option<(u32, u32)> {
    let version = out.trim().strip_prefix("tmux ")?;
    let version = version.strip_prefix("next-").unwrap_or(version);
    let (major, rest) = version.split_once('.')?;
    let minor_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    Some((major.parse().ok()?, rest[..minor_end].parse().ok()?))
}

/// Ghostty config files other than the bundled one, in the order Ghostty loads them.
pub fn other_ghostty_files(home: &Path, xdg: &Path) -> Vec<PathBuf> {
    let app_support = home.join("Library/Application Support/com.mitchellh.ghostty");
    vec![
        xdg.join("ghostty/config.ghostty"),
        app_support.join("config"),
        app_support.join("config.ghostty"),
    ]
}

/// The lines of a Ghostty config that set a color the theme would otherwise set, as
/// 1-based line numbers and keys. An empty value resets the key, so it is not a color.
pub fn color_overrides(text: &str) -> Vec<(usize, &str)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let key = key.trim();
            let is_color = GHOSTTY_COLOR_KEYS.contains(&key) && !value.trim().is_empty();
            is_color.then_some((index + 1, key))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_color_lines_with_or_without_spaces() {
        let text = "\
font-size = 14
background = #ffffff
foreground=#000000
  cursor-color =  #333333
palette = 0=#1e1e1e
palette=15=#ffffff
";
        assert_eq!(
            color_overrides(text),
            vec![
                (2, "background"),
                (3, "foreground"),
                (4, "cursor-color"),
                (5, "palette"),
                (6, "palette"),
            ]
        );
    }

    #[test]
    fn skips_comments_resets_blank_lines_and_similar_keys() {
        let text = "\
# background = #ffffff
  # foreground = #000000

background =
background-opacity = 0.9
selection-foreground-extra = #fff
theme = dark:Catppuccin Mocha
";
        assert!(color_overrides(text).is_empty());
    }

    #[test]
    fn saves_the_mode_as_a_word_and_follows_the_system_without_it() {
        let dir = std::env::temp_dir().join(format!("f0-mode-{}", std::process::id()));
        let file = mode_file(&dir.join("f0"));
        assert_eq!(read_mode(&file), Mode::System);

        write_mode(&file, Mode::Dark).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "dark\n");
        assert_eq!(read_mode(&file), Mode::Dark);
        write_mode(&file, Mode::Light).unwrap();
        assert_eq!(read_mode(&file), Mode::Light);

        write_mode(&file, Mode::System).unwrap();
        assert!(!file.exists());
        write_mode(&file, Mode::System).unwrap();

        std::fs::write(&file, "sepia").unwrap();
        assert_eq!(read_mode(&file), Mode::System);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cycles_through_the_three_modes() {
        assert_eq!(Mode::System.next(), Mode::Light);
        assert_eq!(Mode::Light.next(), Mode::Dark);
        assert_eq!(Mode::Dark.next(), Mode::System);
    }

    #[test]
    fn lists_ghostty_themes_that_are_missing_or_broken() {
        let dir = std::env::temp_dir().join(format!("f0-themes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(missing_ghostty_themes(&dir), ["f0-light", "f0-dark"]);

        std::fs::write(dir.join("f0-light"), "background = #ffffff").unwrap();
        std::os::unix::fs::symlink(dir.join("gone"), dir.join("f0-dark")).unwrap();
        assert_eq!(missing_ghostty_themes(&dir), ["f0-dark"]);

        std::fs::remove_file(dir.join("f0-dark")).unwrap();
        std::fs::write(dir.join("f0-dark"), "background = #1f1f1f").unwrap();
        assert!(missing_ghostty_themes(&dir).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parses_tmux_versions() {
        assert_eq!(tmux_version("tmux 3.8\n"), Some((3, 8)));
        assert_eq!(tmux_version("tmux 3.5a"), Some((3, 5)));
        assert_eq!(tmux_version("tmux next-3.8"), Some((3, 8)));
        assert_eq!(tmux_version("tmux 10.0"), Some((10, 0)));
        assert_eq!(tmux_version("tmux master"), None);
        assert_eq!(tmux_version(""), None);
    }

    #[test]
    fn recognizes_links_into_an_f0_repo() {
        let tmp = std::env::temp_dir().join(format!("f0-appearance-{}", std::process::id()));
        let repo = tmp.join("f0");
        std::fs::create_dir_all(repo.join("crates/f0")).unwrap();
        std::fs::create_dir_all(repo.join("config/tmux")).unwrap();
        std::fs::create_dir_all(tmp.join("other/config/tmux")).unwrap();
        let rel = Path::new("config/tmux");

        let bundled = tmp.join("bundled");
        std::os::unix::fs::symlink(repo.join(rel), &bundled).unwrap();
        let relative = tmp.join("relative");
        std::os::unix::fs::symlink("f0/config/tmux", &relative).unwrap();
        let foreign = tmp.join("foreign");
        std::os::unix::fs::symlink(tmp.join("other/config/tmux"), &foreign).unwrap();
        let plain = tmp.join("plain");
        std::fs::create_dir_all(&plain).unwrap();

        assert!(is_bundled(&bundled, rel));
        assert!(is_bundled(&relative, rel));
        assert!(!is_bundled(&foreign, rel));
        assert!(!is_bundled(&plain, rel));
        assert!(!is_bundled(&tmp.join("missing"), rel));
        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
