#!/bin/sh
# Paints the panes with the f0 colours of the macOS appearance: background,
# text and the 16 ANSI colours. Shells and the agent then match Neovim in any terminal,
# whatever colours the terminal itself has. It also forces tmux's own theme, because
# terminals that do not report theirs (like WezTerm) never switch it.
#
# A mode picked in the f0 launcher (`t`), saved as `light` or `dark` in
# ~/.config/f0/appearance, wins over the macOS appearance.
#
# tmux runs it often (see tmux.conf), so it only writes when the appearance changed.
# Without a picked mode or `defaults` (not macOS) it does nothing and tmux keeps
# `theme detect`.

picked="$(cat "${F0_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/f0}/appearance" 2>/dev/null)"
case "$picked" in
  light | dark) mode="$picked" ;;
  *)
    [ -x /usr/bin/defaults ] || exit 0
    case "$(/usr/bin/defaults read -g AppleInterfaceStyle 2>/dev/null)" in
      Dark) mode=dark ;;
      *) mode=light ;;
    esac
    ;;
esac
[ "$(tmux show-options -gqv @f0-appearance)" = "$mode" ] && exit 0

# The same values as config/ghostty/themes/f0-dark and f0-light.
if [ "$mode" = dark ]; then
  fg="#d4d4d4" bg="#1f1f1f"
  palette="#3a3a3a #f14c4c #8cc265 #d7ba7d #4daafc #c586c0 #4ec9b0 #b0b0b0
    #767676 #ff6b6b #a4d68a #e5cc95 #75beff #d6a2d2 #6fd8c4 #e0e0e0"
else
  fg="#1f1f1f" bg="#ffffff"
  palette="#3b3b3b #c72e2e #3d7a1f #8a6a00 #005fb8 #af00db #0f7a8a #a0a0a0
    #6e6e6e #e04848 #4f9a2c #a67f00 #2a7fd4 #c13fe0 #1a8fa0 #c4c4c4"
fi

set -- set-option -g theme "$mode" \; set-option -g window-style "fg=$fg,bg=$bg"
i=0
for colour in $palette; do
  set -- "$@" \; set-option -g "pane-colours[$i]" "$colour"
  i=$((i + 1))
done
tmux "$@" \; set-option -g @f0-appearance "$mode"
