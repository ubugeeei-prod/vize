#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/check-nix/scripts/keep-current-build-dependencies-through-cache-gc.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
nix build --no-update-lock-file --out-link "$RUNNER_TEMP/vize-nix-cache-root" \
  .#packages.x86_64-linux.vize \
  .#packages.x86_64-linux.vize.cargoArtifacts \
  .#checks.x86_64-linux.shell
