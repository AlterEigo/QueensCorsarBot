#!/usr/bin/fish

set logfile (date +%d-%m-%Y_%H-%M).txt

echo "[restart.fish] Killing possible running instance"
pkill queens_corsar
echo "[restart.fish] Waiting for 2 seconds just in case"
sleep 2
echo "[restart.fish] Bootstrapping new instance"
nohup ./queens_corsar >"$logfile" & disown
echo "[restart.fish] Bot launched in daemon mode"
