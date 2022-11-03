#!/usr/bin/fish

echo "[start.sh] Bootstrapping queens_corsar bot"
./queens_corsar & disown
echo "[start.sh] Bot launched in daemon mode"
