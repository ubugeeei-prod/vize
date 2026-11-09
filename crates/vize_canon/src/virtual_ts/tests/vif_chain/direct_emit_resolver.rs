//! Uses that keep the direct emit-props call share one resolver declaration.
use super::repeated_contracts::generated;
use vize_carton::cstr;

#[test]
fn repeated_direct_emit_resolver_is_emitted_once() {
    // A multi-line value keeps the direct call, which shares one resolver.
    let script = "import KeyPad from './KeyPad.vue'\nconst kind = 'k0'\nconst a0 = 'a0'\nconst onPress = (key: string) => key\n";
    let use_site = "<KeyPad :label=\"[\n  a0,\n][0]\" @press=\"onPress\" />";
    let template = cstr!(
        "<div><template v-if=\"kind === 'k0'\">{use_site}{use_site}</template><template v-else-if=\"kind === 'k1'\">{use_site}{use_site}</template></div>"
    );
    for hoist in [false, true] {
        let output = generated(script, &template, hoist);
        let factory = if hoist {
            "import(\"vize:virtual-ts\").$EmitPropsFactory<typeof KeyPad>"
        } else {
            "$EmitPropsFactory<typeof KeyPad>"
        };
        assert_eq!(
            output.code.matches(factory).count(),
            1,
            "one component emits the emit-resolver instance alias once; hoist={hoist}: {}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches("type __KeyPad_0_emit_resolver = ")
                .count(),
            1,
            "the first visible resolver is declared once; hoist={hoist}: {}",
            output.code
        );
        for idx in 1..4 {
            let alias = cstr!("type __KeyPad_{idx}_emit_resolver = __KeyPad_0_emit_resolver;");
            assert_eq!(
                output
                    .code
                    .lines()
                    .filter(|line| line.trim() == alias.as_str())
                    .count(),
                1,
                "later use {idx} aliases the first visible resolver; hoist={hoist}: {}",
                output.code
            );
        }
        assert_eq!(
            output.code.matches("const __vize_emit_factory_").count(),
            0,
            "multi-line uses keep the direct call; hoist={hoist}: {}",
            output.code
        );
    }
}
