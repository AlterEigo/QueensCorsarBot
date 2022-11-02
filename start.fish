#/usr/bin/fish

set executable "queens_corsar"

echo "[start.sh] Bootstrapping queens_corsar bot"
nohup ./queens_corsar >"$(date "+%d-%m-%Y_%H-%M").txt" & disown
echo "[start.sh] Bot launched in daemon mode"
