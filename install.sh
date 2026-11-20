#!/usr/bin/env bash
# Installs f0: tmux, treehouse, Rust when cargo is missing, the CLI and f0.app with its
# font, plus, optionally, the bundled Neovim, tmux and Ghostty configs.
# Safe to run again: existing f0 links are kept, any other config is only replaced
# after asking (or with --yes), with a backup first, and f0.app is rebuilt from the
# installed Ghostty.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install.sh [options]

  --no-nvim          keep your own Neovim config
  --no-tmux-config   keep your own tmux config
  --no-ghostty       keep your own Ghostty config (f0.app still installs Ghostty
                     and JetBrainsMono Nerd Font)
  --no-app           skip /Applications/f0.app, its Ghostty and its font
  --cli-only         same as the four above: only tmux, treehouse, Rust and the f0 CLI
  --yes              replace existing configs without asking (they are backed up)
  -h, --help         show this help
EOF
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
# ensure_cargo may extend PATH for this run; the warning at the end checks the user's own.
USER_PATH="$PATH"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
STAMP="$(date +%Y%m%d%H%M%S)"
GHOSTTY_APP=/Applications/Ghostty.app
APP=/Applications/f0.app
APP_ID=com.falcondpr.f0
FONT_CASK=font-jetbrains-mono-nerd-font

with_nvim=1
with_tmux=1
with_ghostty=1
with_app=1
assume_yes=0

for arg in "$@"; do
  case "$arg" in
    --no-nvim) with_nvim=0 ;;
    --no-tmux-config) with_tmux=0 ;;
    --no-ghostty) with_ghostty=0 ;;
    --no-app) with_app=0 ;;
    --cli-only) with_nvim=0 with_tmux=0 with_ghostty=0 with_app=0 ;;
    --yes) assume_yes=1 ;;
    -h | --help) usage; exit 0 ;;
    *) echo "unknown option: $arg" >&2; usage >&2; exit 1 ;;
  esac
done

info() { printf '\033[1m==>\033[0m %s\n' "$*"; }

confirm() {
  [ "$assume_yes" = 1 ] && return 0
  local answer
  read -r -p "$1 [y/N] " answer </dev/tty || return 1
  [[ "$answer" =~ ^[Yy]$ ]]
}

link() {
  local src="$1" dst="$2"
  if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
    info "ok      $dst"
    return
  fi
  if [ -e "$dst" ] || [ -L "$dst" ]; then
    if ! confirm "$dst already exists. Replace it with the f0 config (a backup is kept)?"; then
      info "keep    $dst"
      return
    fi
    mv "$dst" "$dst.bak.$STAMP"
    info "backup  $dst -> $dst.bak.$STAMP"
  fi
  mkdir -p "$(dirname "$dst")"
  ln -s "$src" "$dst"
  info "link    $dst -> $src"
}

brew_formula() {
  brew list --formula "$1" >/dev/null 2>&1 || brew install "$1"
}

# A rustup install that is not in PATH yet (new shell not opened) still counts.
ensure_cargo() {
  command -v cargo >/dev/null && return
  if [ -x "$CARGO_BIN/cargo" ]; then
    export PATH="$CARGO_BIN:$PATH"
    return
  fi
  info "installing Rust to build f0"
  brew_formula rust
  if ! command -v cargo >/dev/null; then
    echo "cargo is still not in PATH after installing Rust; open a new shell and run install.sh again" >&2
    exit 1
  fi
}

# Both Ghostty configs set this font, and the launcher and Neovim draw its Nerd Font
# glyphs. A copy installed by hand, without Homebrew, counts too.
ensure_font() {
  if compgen -G "$HOME/Library/Fonts/JetBrainsMonoNerdFont-*" >/dev/null ||
    compgen -G "/Library/Fonts/JetBrainsMonoNerdFont-*" >/dev/null; then
    info "ok      JetBrainsMono Nerd Font"
    return
  fi
  brew install --cask "$FONT_CASK"
}

# f0.app is a copy of Ghostty.app renamed to f0, with its own bundle id, icon and
# executable (crates/f0-app), signed ad hoc. Its executable makes Ghostty read only
# ~/.config/f0/ghostty.conf. Rebuilt from scratch every run, so a newer Ghostty or f0
# reaches it.
build_app() {
  if [ -d "$APP" ]; then
    local id
    id="$(defaults read "$APP/Contents/Info" CFBundleIdentifier 2>/dev/null || true)"
    if [ "$id" != "$APP_ID" ]; then
      echo "$APP exists and is not f0's app (bundle id: ${id:-none}); move it away and run install.sh again" >&2
      exit 1
    fi
    if pgrep -f "$APP/Contents/MacOS/" >/dev/null; then
      info "warning: f0.app is open; quit it and open it again to use the new build"
    fi
    rm -rf "$APP"
  fi
  info "creating $APP from Ghostty $(defaults read "$GHOSTTY_APP/Contents/Info" CFBundleShortVersionString)"
  ditto "$GHOSTTY_APP" "$APP"
  local plist="$APP/Contents/Info.plist"
  /usr/libexec/PlistBuddy \
    -c "Set :CFBundleIdentifier $APP_ID" \
    -c "Set :CFBundleName f0" \
    -c "Set :CFBundleExecutable f0" \
    -c "Set :CFBundleIconFile f0" \
    "$plist"
  /usr/libexec/PlistBuddy -c "Set :CFBundleDisplayName f0" "$plist" 2>/dev/null ||
    /usr/libexec/PlistBuddy -c "Add :CFBundleDisplayName string f0" "$plist"
  # Without it Finder shows the Ghostty icon from the asset catalog instead of f0.icns.
  /usr/libexec/PlistBuddy -c "Delete :CFBundleIconName" "$plist" 2>/dev/null || true
  cp "$ROOT/target/release/f0-app" "$APP/Contents/MacOS/f0"
  cp "$ROOT/config/app/f0.icns" "$APP/Contents/Resources/f0.icns"
  # The themes travel with the app, so it keeps working when the f0 repo moves or is on
  # a commit without them.
  mkdir -p "$APP/Contents/Resources/f0-themes"
  cp "$ROOT/config/ghostty/themes/f0-light" "$ROOT/config/ghostty/themes/f0-dark" \
    "$APP/Contents/Resources/f0-themes/"
  # A Ghostty downloaded with a browser is quarantined, and Gatekeeper would refuse the
  # copy once it is signed ad hoc.
  xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true
  codesign --force --deep --sign - "$APP" 2>/dev/null
  /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP" || true
}

# Fills the paths into config/ghostty/f0.conf. The file is generated, so a different
# one is backed up and replaced without asking.
write_app_config() {
  local dst="$CONFIG/f0/ghostty.conf" tmp
  tmp="$(mktemp)"
  sed -e "s|@F0_BIN@|$(sed_escape "$CARGO_BIN/f0")|g" \
      -e "s|@F0_ICON@|$(sed_escape "$APP/Contents/Resources/f0.icns")|g" \
      -e "s|@F0_THEMES@|$(sed_escape "$APP/Contents/Resources/f0-themes")|g" \
      "$ROOT/config/ghostty/f0.conf" >"$tmp"
  if [ -f "$dst" ] && cmp -s "$tmp" "$dst"; then
    rm "$tmp"
    info "ok      $dst"
    return
  fi
  if [ -f "$dst" ]; then
    mv "$dst" "$dst.bak.$STAMP"
    info "backup  $dst -> $dst.bak.$STAMP"
  fi
  mv "$tmp" "$dst"
  info "write   $dst"
}

sed_escape() {
  printf '%s' "$1" | sed 's/[&|\\]/\\&/g'
}

if [ "$with_app" = 1 ] && [ "$(uname -s)" != Darwin ]; then
  info "f0.app is macOS only; skipping it"
  with_app=0
fi

if ! command -v brew >/dev/null; then
  echo "Homebrew is required: https://brew.sh" >&2
  exit 1
fi

ensure_cargo

brew_formula tmux
# treehouse may come from its own installer instead of Homebrew; any copy in PATH works.
command -v treehouse >/dev/null || brew install treehouse
if [ "$with_nvim" = 1 ]; then
  for formula in neovim ripgrep fd; do
    brew_formula "$formula"
  done
fi
# f0.app is built from Ghostty, so it needs Ghostty even with --no-ghostty.
if { [ "$with_ghostty" = 1 ] || [ "$with_app" = 1 ]; } && [ ! -d "$GHOSTTY_APP" ]; then
  brew install --cask ghostty
fi
if [ "$with_ghostty" = 1 ] || [ "$with_app" = 1 ]; then ensure_font; fi

if [ "$with_nvim" = 1 ]; then link "$ROOT/config/nvim" "$CONFIG/nvim"; fi
if [ "$with_tmux" = 1 ]; then link "$ROOT/config/tmux" "$CONFIG/tmux"; fi
if [ "$with_ghostty" = 1 ]; then
  link "$ROOT/config/ghostty/config" "$CONFIG/ghostty/config"
  # One link per theme, so themes of your own in the same folder stay.
  for theme in f0-light f0-dark; do
    link "$ROOT/config/ghostty/themes/$theme" "$CONFIG/ghostty/themes/$theme"
  done
fi

mkdir -p "$CONFIG/f0/projects"

info "building f0"
cargo install --path "$ROOT/crates/f0" --locked --quiet

if [ "$with_app" = 1 ]; then
  cargo build --release --locked --quiet --manifest-path "$ROOT/Cargo.toml" -p f0-app
  build_app
  write_app_config
fi

case ":$USER_PATH:" in
  *":$CARGO_BIN:"*) ;;
  *) info "warning: $CARGO_BIN, where f0 is installed, is not in PATH; add this to your shell profile:"
     printf '    export PATH="%s:$PATH"\n' "$CARGO_BIN" ;;
esac

# A tmux server that was already running gets the new key bindings without a reload.
if tmux list-sessions >/dev/null 2>&1; then
  info "binding f0 keys in the running tmux server"
  "$CARGO_BIN/f0" keys || true
fi

info "done, run \`f0 doctor\`"
