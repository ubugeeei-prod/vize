#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt3-module-build/download-chromium-dependencies-while-building-the-current-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
archive_dir=
cache_config=
download_config=
cli_pid=
cleanup() {
  status=$?
  trap - EXIT INT TERM
  if [[ -n "$cli_pid" ]]; then
    kill -TERM -- "-$cli_pid" 2>/dev/null || :
    wait "$cli_pid" 2>/dev/null || :
  fi
  for owned_config in "$download_config" "$cache_config"; do
    if [[ -n "$owned_config" ]] && ! sudo rm -f -- "$owned_config"; then
      status=1
    fi
  done
  if [[ -n "$archive_dir" ]] && ! sudo rm -rf -- "$archive_dir"; then
    status=1
  fi
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

archive_dir=$(sudo mktemp -d /var/cache/apt/vize-nuxt-archives-XXXXXX)
sudo chmod 0755 "$archive_dir"
sudo install -m 0755 "$VIZE_CI_FILES/report-apt-archive-inventory.sh" "$archive_dir/report-inventory"
cache_config=$(sudo mktemp /etc/apt/apt.conf.d/99-vize-nuxt-archives-XXXXXX)
download_config=$(sudo mktemp /etc/apt/apt.conf.d/98-vize-nuxt-download-only-XXXXXX)
sudo tee "$cache_config" >/dev/null <<APT
Dir::Cache::archives "$archive_dir";
APT::Update::Post-Invoke { "'$archive_dir/report-inventory'"; };
APT
sudo tee "$download_config" >/dev/null <<'APT'
APT::Get::Download-Only "true";
APT

setsid --wait cargo build --profile ci -p vize &
cli_pid=$!
download_status=0
if (cd tests && vize-ci-apt-retry vp exec playwright install-deps chromium); then
  :
else
  download_status=$?
fi
cli_status=0
if wait "$cli_pid"; then
  :
else
  cli_status=$?
fi
cli_pid=
sudo rm -- "$download_config"
download_config=
printf 'Nuxt3 setup statuses: download=%s CLI=%s\n' "$download_status" "$cli_status"
if (( download_status != 0 )); then
  exit "$download_status"
fi
if (( cli_status != 0 )); then
  exit "$cli_status"
fi
sudo "$archive_dir/report-inventory" before-install
(cd tests && vize-ci-apt-retry vp exec playwright install --with-deps chromium)
