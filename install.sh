#!/usr/bin/env bash
# Installs f0: tmux, treehouse and the CLI, plus, optionally, the bundled Neovim, tmux
# and Ghostty configs.
# Safe to run again: existing f0 links are kept, and any other config is only
# replaced after asking (or with --yes), with a backup first.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install.sh [options]

  --no-nvim          keep your own Neovim config
  --no-tmux-config   keep your own tmux config
  --no-ghostty       keep your own Ghostty config
  --cli-only         same as the three above: only tmux, treehouse and the f0 CLI
  --yes              replace existing configs without asking (they are backed up)
  -h, --help         show this help
EOF
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
STAMP="$(date +%Y%m%d%H%M%S)"

with_nvim=1
with_tmux=1
with_ghostty=1
assume_yes=0

for arg in "$@"; do
  case "$arg" in
    --no-nvim) with_nvim=0 ;;
    --no-tmux-config) with_tmux=0 ;;
    --no-ghostty) with_ghostty=0 ;;
    --cli-only) with_nvim=0 with_tmux=0 with_ghostty=0 ;;
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

if ! command -v brew >/dev/null; then
  echo "Homebrew is required: https://brew.sh" >&2
  exit 1
fi

brew_formula tmux
# treehouse may come from its own installer instead of Homebrew; any copy in PATH works.
command -v treehouse >/dev/null || brew install treehouse
if [ "$with_nvim" = 1 ]; then
  for formula in neovim ripgrep fd; do
    brew_formula "$formula"
  done
fi
if [ "$with_ghostty" = 1 ] && [ ! -d /Applications/Ghostty.app ]; then
  brew install --cask ghostty
fi

if [ "$with_nvim" = 1 ]; then link "$ROOT/config/nvim" "$CONFIG/nvim"; fi
if [ "$with_tmux" = 1 ]; then link "$ROOT/config/tmux" "$CONFIG/tmux"; fi
if [ "$with_ghostty" = 1 ]; then link "$ROOT/config/ghostty/config" "$CONFIG/ghostty/config"; fi

mkdir -p "$CONFIG/f0/projects"

info "building f0"
cargo install --path "$ROOT/crates/f0" --locked --quiet

info "done, run \`f0 doctor\`"
