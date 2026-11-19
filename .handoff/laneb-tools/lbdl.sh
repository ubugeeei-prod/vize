#!/bin/bash
# Download instruction-count artifacts for the given run ids into scratchpad/laneb-ic/<run>.
out=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/laneb-ic
mkdir -p "$out"
for r in "$@"; do
  [ -d "$out/$r" ] || gh run download "$r" -R ubugeeei-prod/vize -D "$out/$r" >/dev/null 2>&1 || echo "download failed $r"
done
