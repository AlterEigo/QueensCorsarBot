#!/usr/bin/fish

echo "[restart.fish] Killing possible running instance"
pkill queens_corsar
echo "[restart.fish] Waiting for 2 seconds just in case"
sleep 2
echo "[restart.fish] Moving log files into logs directory"
for file in (ls *.txt)
	if not lsof $file
		mv $file -t logs/
	end
end
echo "[restart.fish] Bootstrapping new instance"
./queens_corsar & disown
echo "[restart.fish] Bot launched in daemon mode"
