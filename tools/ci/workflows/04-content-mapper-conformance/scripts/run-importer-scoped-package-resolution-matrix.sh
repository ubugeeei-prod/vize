#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-content-mapper-conformance/scripts/run-importer-scoped-package-resolution-matrix.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize --test integration -- check_importer_scoped_package_cli:: --nocapture
cargo test -p vize --test integration -- check_package_declaration_barrel_cli:: --nocapture
cargo test -p vize --test integration -- check_package_paths_authority_cli:: --nocapture
cargo test -p vize --test integration -- check_package_resolution_modes_cli:: --nocapture
cargo test -p vize --test integration -- check_package_scope_mode_cli:: --nocapture
cargo test -p vize --test integration -- check_package_tsx_shadow_cli:: --nocapture
