use super::{assert_success, run, write};
use std::fs;

#[test]
fn object_wrap_config_and_cli_overrides_format_the_reported_source() {
    let source = "const options = {\n a:1,\n};\n";
    for (configured, flags, expected) in [
        ("collapse", &[][..], "const options = { a: 1 };\n"),
        (
            "collapse",
            &["--object-wrap", "preserve"][..],
            "const options = {\n  a: 1,\n};\n",
        ),
        (
            "preserve",
            &["--object-wrap", "collapse"][..],
            "const options = { a: 1 };\n",
        ),
    ] {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        let config =
            vize_l0::cstr!("export default {{ fmt: {{ objectWrap: \"{configured}\" }} }};\n");
        write(root, "vite.config.mjs", &config);
        for (file, original, expected) in [
            ("case.ts", source.into(), expected.into()),
            (
                "Case.vue",
                vize_l0::cstr!("<script setup lang=\"ts\">\n{source}</script>\n"),
                vize_l0::cstr!("<script setup lang=\"ts\">\n{expected}</script>\n"),
            ),
        ] {
            write(root, file, &original);
            let mut args = vec!["fmt", "--write", file];
            args.extend_from_slice(flags);
            for _ in 0..2 {
                assert_success(&run(root, &args));
                assert_eq!(
                    fs::read_to_string(root.join(file)).unwrap(),
                    expected.as_str()
                );
            }
            let mut check = vec!["fmt", "--check", file];
            check.extend_from_slice(flags);
            assert_success(&run(root, &check));
        }
    }
}
