#!/usr/bin/fish

set executable "queens_corsar"
set logfile (date +%d-%m-%Y_%H-%M).txt

echo "[start.sh] Bootstrapping queens_corsar bot"
nohup ./queens_corsar >"$logfile" & disown
echo "[start.sh] Bot launched in daemon mode"
