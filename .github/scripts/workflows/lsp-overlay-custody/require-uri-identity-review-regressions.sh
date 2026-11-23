#!/usr/bin/env bash
set -euo pipefail
test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE"
capture="$RUNNER_TEMP/lsp-overlay-custody/identity-review"
mkdir -p "$capture"
git rev-parse HEAD > "$capture/source.txt"
set +e
cargo test --locked --profile ci -p vize_canon --lib -- lsp_client::editor_lsp::client::tests::hierarchy_items_preserve_unmapped_foreign_documents lsp_client::session::tests::moving_one_spelling_preserves_the_other_native_identity_owner lsp_client::session::tests::releasing_either_normalized_owner_keeps_the_other_until_the_last_close lsp_client::utils::tests::converging_workspace_edit_keys_keep_every_edit_in_each_original_order lsp_client::editor_lsp::file_rename::tests::materialized_renames_share_one_native_namespace_and_keep_each_caller_context lsp_client::lifecycle::workspace::initial_project_tests::native_first_materialized_editor_matches_existing_activation_and_delta lsp_client::session::logical::tests::an_independent_non_authored_owner_is_not_promoted_to_logical_authority lsp_client::session::logical::tests::a_missing_attached_snapshot_is_an_error_not_accepted_logical_authority --nocapture > "$capture/canon.stdout" 2> "$capture/canon.stderr"
canon_status=$?
printf '%s\n' "$canon_status" > "$capture/canon.status"
cargo test --locked --profile ci -p vize_maestro --lib -- ide::corsa_support::authored_uri::tests::closed_foreign_targets_keep_the_import_path_before_an_open_owner_takes_precedence --nocapture > "$capture/maestro.stdout" 2> "$capture/maestro.stderr"
maestro_status=$?
printf '%s\n' "$maestro_status" > "$capture/maestro.status"
set -e
test "$canon_status" = 0
test "$maestro_status" = 0
