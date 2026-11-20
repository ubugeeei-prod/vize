set -euo pipefail

[[ "$(git rev-parse HEAD)" == "$SOURCE_SHA" ]]
cargo build --locked --profile ci -p vize --bin vize
