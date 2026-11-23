#!/usr/bin/env bash
set -euo pipefail
[[ "$RELEASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
mkdir -p "$PROJECT/app" "$EVIDENCE"
bun "$VIZE_CI_FILES/install-build-and-serve-the-exact-public-nuxt-integration-with-bun.ts"

install_started=$(date +%s%N)
(cd "$PROJECT" && bun install --no-progress)
install_ms=$(( ($(date +%s%N) - install_started) / 1000000 ))
build_started=$(date +%s%N)
(cd "$PROJECT" && bun run build)
build_ms=$(( ($(date +%s%N) - build_started) / 1000000 ))
mkdir -p "$DEPLOY"
cp -R "$PROJECT/.output/server" "$DEPLOY/server"
cp -R "$PROJECT/.output/public" "$DEPLOY/public"
runtime_install_started=$(date +%s%N)
(cd "$DEPLOY/server" && bun install --no-progress --production)
runtime_install_ms=$(( ($(date +%s%N) - runtime_install_started) / 1000000 ))
cp "$DEPLOY/server/package.json" "$EVIDENCE/server-package.json"

server_pid=
cleanup() {
  status=$?
  trap - EXIT
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || :
    wait "$server_pid" 2>/dev/null || :
  fi
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
(
  cd "$DEPLOY/server"
  NITRO_HOST=127.0.0.1 NITRO_PORT=3210 bun index.mjs
) > "$EVIDENCE/server.log" 2>&1 &
server_pid=$!
for _ in $(seq 1 60); do
  if curl --fail --silent --show-error http://127.0.0.1:3210/ \
    > "$EVIDENCE/response.html"; then
    break
  fi
  sleep 0.5
done
grep -F 'vize-public-nuxt-bun' "$EVIDENCE/response.html"

INSTALL_MS="$install_ms" BUILD_MS="$build_ms" \
  RUNTIME_INSTALL_MS="$runtime_install_ms" bun "$VIZE_CI_FILES/install-build-and-serve-the-exact-public-nuxt-integration-with-bun-2.ts"
