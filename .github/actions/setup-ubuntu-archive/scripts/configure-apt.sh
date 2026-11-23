#!/usr/bin/env bash
set -euo pipefail
sudo tee /etc/apt/apt.conf.d/80-vize-retry-downloads >/dev/null <<'APT'
Acquire::Retries "5";
Acquire::http::Timeout "30";
Acquire::https::Timeout "30";
Acquire::Queue-Mode "access";
Acquire::http::Pipeline-Depth "0";
Acquire::https::Pipeline-Depth "0";
APT

sudo tee /usr/local/bin/vize-ci-apt-retry >/dev/null < "$GITHUB_ACTION_PATH/scripts/retry-apt.sh"
sudo chmod +x /usr/local/bin/vize-ci-apt-retry

mirror_list=/etc/apt/blacksmith-ubuntu-mirrors.txt
if [[ -f "$mirror_list" ]]; then
  printf '%s\n' 'https://archive.ubuntu.com/ubuntu' | sudo tee "$mirror_list" >/dev/null
fi

apt_source_files=()
for candidate in /etc/apt/sources.list /etc/apt/sources.list.d/*.list /etc/apt/sources.list.d/*.sources; do
  [[ -f "$candidate" ]] && apt_source_files+=("$candidate")
done
if (( ${#apt_source_files[@]} > 0 )); then
  sudo sed -i \
    -f "$GITHUB_ACTION_PATH/scripts/secure-ubuntu-mirrors.sed" \
    "${apt_source_files[@]}"
fi
