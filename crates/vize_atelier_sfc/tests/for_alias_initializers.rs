//! #10746: loop alias initializers read component context, not free globals.
#![expect(clippy::disallowed_types, reason = "complete compiler output")]

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

fn compile(source: &str, ssr: bool) -> std::string::String {
    let parse = SfcParseOptions {
        filename: "App.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("valid SFC");
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.ssr = ssr;
    let result = compile_sfc(&descriptor, options).expect("compiled SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code.to_string()
}

fn template(script: &str, body: &str, newline: &str) -> std::string::String {
    [
        script,
        newline,
        "<template>",
        newline,
        body,
        newline,
        "</template>",
        newline,
    ]
    .concat()
}

const OPTIONS: &str = "<script>export default {}</script>";
const SETUP: &str = "<script setup>const fallback = 'A'; const items = [{}]</script>";

#[test]
fn initializers_and_computed_keys_read_component_context() {
    for newline in ["\n", "\r\n"] {
        for (body, ssr, vdom) in [
            (
                "<p v-for=\"{ label = fallback } in items\">{{ label }}</p>",
                "({ label = _ctx.fallback }) =>",
                "({ label = _ctx.fallback }) =>",
            ),
            (
                "<p v-for=\"{ \\u006cabel = fallback } in items\">{{ label }}</p>",
                "({ \\u006cabel = _ctx.fallback }) =>",
                "({ \\u006cabel = _ctx.fallback }) =>",
            ),
            (
                "<p v-for=\"[a = fallback, b = a] of items\">{{ a }}{{ b }}</p>",
                "([a = _ctx.fallback, b = a]) =>",
                "([a = _ctx.fallback, b = a]) =>",
            ),
            (
                "<p v-for=\"{ [key]: { name = pick(name, other) } } in items\">{{ name }}</p>",
                "({ [_ctx.key]: { name = _ctx.pick(name, _ctx.other) } }) =>",
                "({ [_ctx.key]: { name = _ctx.pick(name, _ctx.other) } }) =>",
            ),
            (
                "<div v-for=\"outer in rows\"><p v-for=\"{ label = outer.name } in outer.items\">{{ label }}</p></div>",
                "({ label = outer.name }) =>",
                "({ label = outer.name }) =>",
            ),
            (
                "<p v-for=\"(item, index) in items\">{{ item }}</p>",
                "(item, index) =>",
                "(item, index) =>",
            ),
        ] {
            let source = template(OPTIONS, body, newline);
            assert!(compile(&source, true).contains(ssr), "{body}");
            assert!(compile(&source, false).contains(vdom), "{body}");
        }
    }
}

#[test]
fn setup_bindings_keep_their_render_function_access() {
    let source = template(
        SETUP,
        "<p v-for=\"{ label = fallback } in items\">{{ label }}</p>",
        "\n",
    );
    assert!(compile(&source, true).contains("({ label = $setup.fallback }) =>"));
    // The inline render closure reads the setup constant directly.
    assert!(compile(&source, false).contains("({ label = fallback }) =>"));
}
