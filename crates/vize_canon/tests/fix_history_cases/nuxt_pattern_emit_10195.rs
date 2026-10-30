use crate::support::fix_history_runner::check_pack;

pub(super) fn check() {
    check_pack(
        "content-mapper-nuxt-emit-10195",
        "content_mapper_preserves_emit_with_nuxt_pattern_injections",
        "369f01b50d8fe6e3b0996cc9a90eecf422dbdd3b",
        Some(10195),
        &[
            "nuxt-pattern-injection-attribute-listener",
            "pattern-injections-keep-listener-errors",
        ],
    );
}
