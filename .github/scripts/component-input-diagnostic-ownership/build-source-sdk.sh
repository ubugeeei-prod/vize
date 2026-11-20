set -euo pipefail

cd typescript-content-mapper/tsc
[[ "$(git rev-parse HEAD)" == d6c4afddb2c55f4a9dea7b59293a99a8fdea1799 ]]
capture="$RUNNER_TEMP/component-input-d6c4"
mkdir -p "$capture"
go build -tags=noembed -trimpath -o "$capture/tsgo" ./cmd/tsc
cp internal/bundled/libs/*.d.ts "$capture/"
git rev-parse HEAD > "$capture/source.txt"
sha256sum "$capture/tsgo" > "$capture/binary.sha256"
