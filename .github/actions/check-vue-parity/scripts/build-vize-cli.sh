#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/check-vue-parity/scripts/build-vize-cli.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo_args=(build --profile ci -p vize --features legacy)
if [ "$RUNNER_ENVIRONMENT" = "github-hosted" ]; then
  cargo_args+=(--jobs 2)
fi
cargo "${cargo_args[@]}"
vp exec node tests/differential/build-receipt.ts --legacy
cat target/ci/vize.differential-build.json
oracle_args=(build --profile ci -p vize_atelier_sfc --features legacy --example cli_build_result_oracle)
if [ "$RUNNER_ENVIRONMENT" = "github-hosted" ]; then
  oracle_args+=(--jobs 2)
fi
cargo "${oracle_args[@]}"
