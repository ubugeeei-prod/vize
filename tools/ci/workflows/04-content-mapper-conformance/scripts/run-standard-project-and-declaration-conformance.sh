#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-content-mapper-conformance/scripts/run-standard-project-and-declaration-conformance.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE=standard-go-d6-vue35 VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR="$GITHUB_WORKSPACE/target/vize-tests/content-mapper-directive-owners/reported-dynamic-targets" cargo test -p vize --test integration -- content_mapper_tsgo_cli:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_directives:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_build:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_incremental:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_package_references:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_watch:: --nocapture
cargo test -p vize --test integration -- content_mapper_importer_scoped_packages:: --nocapture
