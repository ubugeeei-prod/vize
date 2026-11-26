#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-content-mapper-conformance/scripts/run-authored-syntax-and-setup-binding-conformance.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize --test integration -- content_mapper_tsgo_cli::template_expression_syntax:: content_mapper_tsgo_cli::reported_model_events_11390_11391:: content_mapper_tsgo_cli::reported_expression_duplicates_11374:: content_mapper_tsgo_cli::dynamic_template_refs::standard_tsgo_static_template_refs_read_runtime_bindings_and_keep_true_unused --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::const_enum_import:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::type_only_import_use:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::standard_tsgo_reports_each_component_prop_problem_once content_mapper_tsgo_cli::fallthrough_value_12458:: content_mapper_tsgo_cli::library_generic_fallthrough_13021:: content_mapper_tsgo_cli::packaged_generic_data_attrs:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::standard_tsgo_contextually_types_multiline_callback_prop --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::unknown_options_10218:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::generic_constructor_slots:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::quoted_generic_parameters:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::long_prop_kebab_case:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::optional_prop_spacing:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_cli::pug_refusal:: --nocapture
