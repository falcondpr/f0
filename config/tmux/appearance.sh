#!/bin/sh
# Paints the panes with the Catppuccin colours of the macOS appearance: background,
# text and the 16 ANSI colours. Shells and the agent then match Neovim in any terminal,
# whatever colours the terminal itself has. It also forces tmux's own theme, because
# terminals that do not report theirs (like WezTerm) never switch it.
#
# tmux runs it often (see tmux.conf), so it only writes when the appearance changed.
# Without `defaults` (not macOS) it does nothing and tmux keeps `theme detect`.

[ -x /usr/bin/defaults ] || exit 0
case "$(/usr/bin/defaults read -g AppleInterfaceStyle 2>/dev/null)" in
  Dark) mode=dark ;;
  *) mode=light ;;
esac
[ "$(tmux show-options -gqv @f0-appearance)" = "$mode" ] && exit 0

if [ "$mode" = dark ]; then
  fg="#cdd6f4" bg="#1e1e2e"
  palette="#45475a #f38ba8 #a6e3a1 #f9e2af #89b4fa #f5c2e7 #94e2d5 #a6adc8
    #585b70 #f37799 #89d88b #ebd391 #74a8fc #f2aede #6bd7ca #bac2de"
else
  fg="#4c4f69" bg="#eff1f5"
  palette="#5c5f77 #d20f39 #40a02b #df8e1d #1e66f5 #ea76cb #179299 #acb0be
    #6c6f85 #de293e #49af3d #eea02d #456eff #fe85d8 #2d9fa8 #bcc0cc"
fi

set -- set-option -g theme "$mode" \; set-option -g window-style "fg=$fg,bg=$bg"
i=0
for colour in $palette; do
  set -- "$@" \; set-option -g "pane-colours[$i]" "$colour"
  i=$((i + 1))
done
tmux "$@" \; set-option -g @f0-appearance "$mode"
