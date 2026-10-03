use super::support::{
    LOCALES, collect, complete, expected, native, parity, registered, selected, span, warning,
};
use vize_l0::Allocator;
use vize_patina::native::NativeSyntaxLint;

// This intentional boundary evidence grants no complete-file equivalence or
// clean credit. Supplied headers contain no host comment/suppression receipt.
fn suppression_gap(source: &str, spelling: &str) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    for locale in LOCALES {
        let mut findings = Vec::new();
        collect(&lint, owner.children(), locale, &mut findings).unwrap();
        assert_eq!(
            expected(findings.iter().map(native).collect()),
            expected(vec![warning(locale, span(source, spelling))])
        );
        assert_eq!(complete(&registered(source, locale)), expected(vec![]));
    }
}

#[test]
fn element_opening_line_suppression_covers_wrapped_full_directive_and_bind() {
    suppression_gap(
        "<template>\n<!-- eslint-disable-next-line vue/no-v-html -->\n<div\n class='a-really-long-class-name'\n v-html='content'\n></div>\n</template>",
        "v-html='content'",
    );
    suppression_gap(
        "<template>\n<!-- eslint-disable-next-line vue/no-v-html -->\n<div\n :innerHTML='content'\n></div>\n</template>",
        "innerHTML",
    );
}

#[test]
fn block_all_rule_and_same_line_comment_controls_remain_host_authority() {
    for source in [
        "<template><!-- eslint-disable vue/no-v-html --><div v-html='x' /></template>",
        "<template><!-- eslint-disable --><div v-html='x' /></template>",
        "<template><div v-html='x' /><!-- eslint-disable-line vue/no-v-html --></template>",
    ] {
        suppression_gap(source, "v-html='x'");
    }
}

#[test]
fn unrelated_comments_and_disabled_rules_do_not_hide_original_sink_warnings() {
    for source in [
        "<template><!-- ordinary comment --><div v-html='x' /></template>",
        "<template>\n<!-- eslint-disable-next-line vue/no-inline-style -->\n<div v-html='x' />\n</template>",
        "<template><div\n class='wrapped'\n v-html='x'\n /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(
                parity(source, locale),
                expected(vec![warning(locale, span(source, "v-html='x'"))])
            );
        }
    }
}
