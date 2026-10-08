//! Existing JSX opt-in remains separate from the new bare read route.
use super::{fixture::Fixture, reads, runtime_enabled};

#[test]
fn jsx_and_tsx_keep_whole_enabled_reads_and_disabled_public_gates() {
    if !runtime_enabled().unwrap() {
        return;
    }
    let source = include_str!(
        "../../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/Unicode.jsx.txt"
    );
    for newline in ["\n", "\r\n"] {
        for (extension, language) in [("jsx", "javascriptreact"), ("tsx", "typescriptreact")] {
            for enabled in [false, true] {
                let name = format!("src/Unicode.{extension}");
                let sources = [(name.as_str(), source.replace('\n', newline))];
                let mut fixture = Fixture::new(&sources, enabled).unwrap();
                fixture.prove_native_diagnostics().unwrap();
                fixture.open_both(&name, &sources[0].1, language, 1);
                reads::identity(&mut fixture, &name, (0, 20, 24), (1, 24, 28), enabled).unwrap();
                if !enabled {
                    fixture
                        .refuse_writes(&name, (0, 20, 24), "caféNext")
                        .unwrap();
                }
                fixture.reopen_both(&name, language, 1).unwrap();
                reads::identity(&mut fixture, &name, (0, 20, 24), (1, 24, 28), enabled).unwrap();
                fixture.assert_disks().unwrap();
                fixture.finish().unwrap();
            }
        }
    }
}
