#!/usr/bin/env bash
# Every 5 min: if free disk < LOW GB, delete cargo target/ dirs of sibling vize worktrees idle > IDLE_H hours,
# largest first, until free >= HIGH GB. Prints one line per deletion. Runs until killed.
LOW=${LOW:-25}; HIGH=${HIGH:-40}; IDLE_H=${IDLE_H:-3}
ROOT=/Users/ubugeeei/Source/github.com/ubugeeei-prod
free_gb() { /bin/df -g / | awk 'NR==2{print $4}'; }
while true; do
  if [ "$(free_gb)" -lt "$LOW" ]; then
    now=$(date +%s)
    for d in $(for t in "$ROOT"/vize.*/target; do [ -d "$t" ] || continue; m=$(stat -f %m "$t"); [ $(( (now - m) / 3600 )) -ge "$IDLE_H" ] || continue; echo "$(/usr/bin/du -sk "$t" | cut -f1) $t"; done | sort -rn | awk '{print $2}'); do
      [ "$(free_gb)" -ge "$HIGH" ] && break
      rm -rf "$d" && echo "watchdog: removed $d (free now $(free_gb) GB) at $(date -u +%H:%MZ)"
    done
    [ "$(free_gb)" -lt "$LOW" ] && echo "watchdog: still low ($(free_gb) GB) after cleanup at $(date -u +%H:%MZ)"
  fi
  sleep 300
done
