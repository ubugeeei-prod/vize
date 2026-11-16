//! TSRX modules on the DOM target (#6858). Each module under
//! `tests/_fixtures/tsrx/mount/` has its committed DOM output (TypeScript: host code is kept) beside it; the
//! mounted-parity test (`tests/tooling/tsrx-mount.test.ts`) runs exactly those
//! bytes against the pinned reference. `VIZE_TSRX_DOM_WRITE=1` rewrites them.
#[cfg(test)]
mod cases {
    use std::path::Path;
    use vize_l4::lang::tsrx::{CompileOptions, Target, compile};

    const MODULES: [&str; 12] = [
        "attributes",
        "counter",
        "default",
        "dynamic",
        "empty",
        "loops",
        "nested",
        "props",
        "returns",
        "slots",
        "typed",
        "values",
    ];

    #[test]
    fn mounted_parity_modules_emit_their_committed_dom_output() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/tsrx/mount");
        let write = std::env::var_os("VIZE_TSRX_DOM_WRITE").is_some();
        for module in MODULES {
            let source = std::fs::read_to_string(root.join(format!("{module}.tsrx"))).unwrap();
            let options = CompileOptions {
                filename: module,
                target: Target::Dom,
                source_map: false,
                erase_types: false,
            };
            let code = match compile(&source, &options) {
                Ok(compiled) => compiled.code,
                Err(failure) => panic!("{module}: {:?}", failure.diagnostics),
            };
            let committed = root.join(format!("{module}.dom.ts"));
            if write {
                std::fs::write(&committed, code.as_str()).unwrap();
            }
            assert_eq!(
                code.as_str(),
                std::fs::read_to_string(&committed).unwrap(),
                "{module}"
            );
        }
    }
}
