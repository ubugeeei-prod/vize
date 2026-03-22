#!/bin/bash
set -euo pipefail

EXPECTED_VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
PUBLISH_RETRY_LIMIT=${PUBLISH_RETRY_LIMIT:-60}
PUBLISH_RETRY_DELAY=${PUBLISH_RETRY_DELAY:-5}

echo "Expected version: $EXPECTED_VERSION"

is_published() {
  local crate=$1
  local version=$2

  curl -sf "https://crates.io/api/v1/crates/$crate" 2>/dev/null | \
    jq -e --arg version "$version" '.versions[] | select(.num == $version)' \
      >/dev/null 2>&1
}

is_resolvable_from_registry() {
  local crate=$1
  local version=$2

  cargo info "$crate@$version" >/dev/null 2>&1
}

wait_for_registry_resolution() {
  local crate=$1
  local version=$2
  local attempt=1

  until is_resolvable_from_registry "$crate" "$version"; do
    if [ "$attempt" -ge "$PUBLISH_RETRY_LIMIT" ]; then
      echo "Timed out waiting for $crate v$version to become resolvable from crates.io."
      return 1
    fi

    echo "Waiting for $crate v$version to become resolvable from crates.io... ($attempt/$PUBLISH_RETRY_LIMIT)"
    attempt=$((attempt + 1))
    sleep "$PUBLISH_RETRY_DELAY"
  done

  echo "$crate v$version is resolvable from crates.io."
}

publish_crate() {
  local crate=$1
  echo "Publishing $crate v$EXPECTED_VERSION..."

  if is_published "$crate" "$EXPECTED_VERSION"; then
    echo "$crate v$EXPECTED_VERSION is already published, skipping publish."
    wait_for_registry_resolution "$crate" "$EXPECTED_VERSION"
    return 0
  fi

  cargo publish -p "$crate"
  echo "$crate v$EXPECTED_VERSION published successfully."
  wait_for_registry_resolution "$crate" "$EXPECTED_VERSION"
}

publish_crate vize_carton
publish_crate vize_relief
publish_crate vize_armature
publish_crate vize_croquis
publish_crate vize_atelier_core
publish_crate vize_atelier_dom
publish_crate vize_atelier_vapor
publish_crate vize_atelier_ssr
publish_crate vize_atelier_sfc
# vize_glyph, vize_vitrine, vize_maestro, vize are skipped:
# vize_glyph depends on oxc_formatter which is not yet on crates.io,
# and the others depend on vize_glyph (even as optional, crates.io still resolves it)
publish_crate vize_canon
publish_crate vize_patina
publish_crate vize_musea
publish_crate vize_fresco

echo "Done!"
