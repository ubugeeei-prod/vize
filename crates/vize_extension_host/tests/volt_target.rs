//! P6-6 — the Volt output-target exercise, end to end.
//!
//! Builds `examples/volt-target` and drives it through the output-target
//! host in both hosting modes. The recorded command is this test.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]

mod support;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use support::build_guest_manifest;
use vize_extension_host::outproc::{OutOfProcessGuest, serve_output_command};
use vize_extension_host::output::{OutputError, OutputSession, OutputTargetGuest};
use vize_extension_host::wasm::WasmOutputGuest;
use vize_extension_host::{GuestError, GuestLimits, Page};
use vize_s0::String;

fn guest() -> &'static Path {
    static GUEST: OnceLock<PathBuf> = OnceLock::new();
    GUEST.get_or_init(|| {
        let manifest =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/volt-target/Cargo.toml");
        build_guest_manifest(
            &manifest,
            "volt-target",
            "vize_contract_volt_target_guest",
            &[],
        )
    })
}

fn both_modes() -> [(&'static str, Box<dyn OutputTargetGuest>); 2] {
    let runner = Path::new(env!("CARGO_BIN_EXE_vize-extension-host"));
    let limits = GuestLimits::default();
    [
        (
            "out-of-process",
            Box::new(
                OutOfProcessGuest::spawn(serve_output_command(runner, guest(), limits))
                    .expect("the guest loads"),
            ),
        ),
        (
            "in-process",
            Box::new(WasmOutputGuest::load_with(guest(), limits).expect("the guest loads")),
        ),
    ]
}

fn page(name: &str) -> Page {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/volt-target/fixtures")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    Page {
        schema_version: 1,
        text: String::from_utf8(bytes).expect("utf-8"),
    }
}

#[test]
fn volt_emits_one_heex_module_in_both_modes() {
    let request = vize_extension_host::output::EmitRequest {
        s2: page("probe.s2.folio"),
        s3: page("probe.s3.folio"),
    };
    let golden = page("probe.emit.folio").text;
    for (mode, guest) in both_modes() {
        let mut session = OutputSession::open(guest).unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert_eq!(session.negotiated().protocol_version, 1, "{mode}");
        assert!(session.negotiated().ignored.is_empty(), "{mode}");
        let accepted = session
            .emit(&request)
            .unwrap_or_else(|e| panic!("{mode}: {e}"));
        assert_eq!(accepted.emitted.document.text, golden, "{mode}");
        assert_eq!(
            accepted.document.text.as_str(),
            "~H\"<p>Hello</p>\"\n",
            "{mode}"
        );
        assert_eq!(accepted.document.links.len(), 1, "{mode}");
        assert_eq!(
            accepted.document.links[0].name.as_deref(),
            Some("hello"),
            "{mode}"
        );
        let link = &accepted.document.links[0];
        let generated =
            &accepted.document.text[link.generated.start as usize..link.generated.end as usize];
        assert_eq!(generated, "Hello", "{mode}");
    }
}

#[test]
fn an_unknown_volt_request_traps_in_both_modes() {
    let request = vize_extension_host::output::EmitRequest {
        s2: page("probe.s2.folio"),
        s3: Page {
            schema_version: 1,
            text: String::from("other\n"),
        },
    };
    for (mode, guest) in both_modes() {
        let mut session = OutputSession::open(guest).expect("negotiates");
        assert_eq!(
            session.emit(&request),
            Err(OutputError::Guest(GuestError::Trap(String::from(
                "wasm trap: wasm `unreachable` instruction executed"
            )))),
            "{mode}"
        );
    }
}
