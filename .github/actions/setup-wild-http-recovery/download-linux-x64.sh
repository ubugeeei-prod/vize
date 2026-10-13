#!/usr/bin/env bash
# Sourced by the pinned original Wild recipe; no downloaded program is run here.
wild_archive=$1
wild_primary_url=$2
wild_retries=$3

if [[ "$RUNNER_OS" != Linux || "$RUNNER_ARCH" != X64 || "$wild_retries" != 5 ||
      "$wild_primary_url" != https://github.com/wild-linker/wild/releases/download/0.9.0/wild-linker-0.9.0-x86_64-unknown-linux-gnu.tar.gz ]]; then
  echo 'Pinned Wild recovery received an unsupported context' >&2
  exit 1
fi

wild_primary_status=0
wild_http=$(
  set +e
  curl --fail-with-body --show-error --location \
       --retry "$wild_retries" --retry-all-errors \
       --output "$wild_archive" --write-out '%{http_code}' \
       "$wild_primary_url"
  wild_curl_status=$?
  printf '|'
  exit "$wild_curl_status"
) || wild_primary_status=$?
wild_http="${wild_http%|}"

case "$wild_primary_status:$wild_http" in
  0:200) ;;
  22:500|22:502|22:503|22:504)
    curl --fail-with-body --show-error --location \
         --header 'Accept: application/octet-stream' \
         --header 'X-GitHub-Api-Version: 2022-11-28' \
         --output "$wild_archive" \
         'https://api.github.com/repos/wild-linker/wild/releases/assets/428152085' || exit "$?"
    ;;
  0:*) echo 'Pinned Wild primary download returned an unsupported HTTP status' >&2; exit 1 ;;
  *) exit "$wild_primary_status" ;;
esac

printf '%s  %s\n' 'deb6ee0e5caec798053ec4aafaba042e20a8edf91f08cb4d36268571cc628d3b' "$wild_archive" |
  sha256sum --check --status || {
    echo 'Pinned Wild archive checksum verification failed' >&2
    exit 1
  }
