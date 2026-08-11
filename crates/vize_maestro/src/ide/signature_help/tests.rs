mod fixture;

use super::{SignatureHelpService, SignatureHelpStage};
use crate::ide::IdeContext;
use fixture::{Fixture, resolve_tsgo_binary};

#[test]
fn signature_help_maps_sfc_script_and_template_with_crlf_and_utf16() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new(&corsa_path);
        let source = "<script lang=\"ts\">\r\nfunction regularFormat(value: string, precision: number): string { return value.repeat(precision) }\r\nregularFormat('🦀', );\r\n</script>\r\n<script setup lang=\"ts\">\r\nconst emoji = '🦀';\r\nfunction format(value: string, precision: number): string { return value.repeat(precision) }\r\nformat(emoji, );\r\n</script>\r\n\r\n<template>\r\n  <p>{{ format('🦀', ) }}</p>\r\n</template>\r\n";
        let (state, uri) = fixture.vue("App.vue", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        for (marker, name) in [
            ("regularFormat('🦀', ", "regularFormat"),
            ("format(emoji, ", "format"),
            ("format('🦀', ", "format"),
        ] {
            let offset = source.find(marker).unwrap() + marker.len();
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let help = SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .expect("signature help at authored SFC call");
            assert_signature(help, name, 1);
        }

        let offset = source.find("const emoji").unwrap() + "const".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        assert!(
            SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .is_none(),
            "signature help must not leak outside a call expression"
        );

        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn signature_help_maps_art_variant_template() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new_traced(&corsa_path);
        let source = "<script setup lang=\"ts\">\nfunction format(value: string, precision: number): string { return value.repeat(precision) }\nformat('script', )\n</script>\n\n<art title=\"Button\" component=\"./Button.vue\">\n  <variant name=\"Empty\">\n    <p>Nothing to call</p>\n  </variant>\n  <variant name=\"Primary\">\n    <p>{{ format('art', ) }}</p>\n  </variant>\n</art>\n";
        let (state, uri) = fixture.vue("Button.art.vue", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        let script_marker = "format('script', ";
        let script_offset = source.find(script_marker).unwrap() + script_marker.len();
        let script_ctx = IdeContext::new(&state, &uri, script_offset).unwrap();
        let (help, stages) = SignatureHelpService::signature_help_with_corsa_traced(
            &script_ctx,
            Some(bridge.clone()),
        )
        .await;
        let help = help.unwrap_or_else(|| {
            panic!(
                "signature help in art script setup: {stages:?}; {}",
                fixture.describe_server_frames()
            )
        });
        assert_eq!(
            stages,
            [
                SignatureHelpStage::VirtualOpened,
                SignatureHelpStage::RequestSome,
            ]
        );
        assert_signature(help, "format", 1);

        let marker = "format('art', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let info = match ctx.block_type {
            Some(crate::virtual_code::BlockType::Art(
                crate::virtual_code::ArtCursorPosition::VariantTemplate(info),
            )) => info,
            other => panic!("expected art variant context, got {other:?}"),
        };
        let template = ctx
            .virtual_docs
            .as_ref()
            .and_then(|docs| docs.art_template(info.variant_index))
            .expect("art variant virtual document");
        assert!(
            template.content.contains("format('art', )"),
            "generated art template lost the authored call:\n{}",
            template.content
        );
        assert!(
            template.content.contains("function format"),
            "generated art template lost the callable declaration:\n{}",
            template.content
        );
        let generated_offset = template
            .source_map
            .to_generated_for(offset as u32, |features| features.signature_help)
            .expect("art signature-help mapping") as usize;
        let expected_generated_offset = template.content.find(marker).unwrap() + marker.len();
        assert_eq!(
            generated_offset, expected_generated_offset,
            "art cursor mapped to the wrong virtual offset:\n{}",
            template.content
        );
        let (help, stages) =
            SignatureHelpService::signature_help_with_corsa_traced(&ctx, Some(bridge.clone()))
                .await;
        let help = help.unwrap_or_else(|| {
            panic!(
                "signature help in art variant: {stages:?}; {}",
                fixture.describe_server_frames()
            )
        });
        assert_eq!(
            stages,
            [
                SignatureHelpStage::VirtualOpened,
                SignatureHelpStage::RequestSome,
            ]
        );
        assert_signature(help, "format", 1);

        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn signature_help_maps_tsx_calls() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new_traced(&corsa_path);
        let source = "function format(value: string, precision: number): string { return value.repeat(precision) }\nexport default () => <p>{format('tsx', )}</p>;\n";
        let (state, uri) = fixture.tsx("Component.tsx", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        let marker = "format('tsx', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let (help, stages) =
            crate::ide::jsx::signature_help_traced(&ctx, Some(bridge.clone()), None).await;
        let help = help.unwrap_or_else(|| {
            panic!(
                "signature help in TSX: {stages:?}; {}",
                fixture.describe_server_frames()
            )
        });
        assert_eq!(
            stages,
            [
                SignatureHelpStage::VirtualOpened,
                SignatureHelpStage::RequestSome,
            ]
        );
        assert_signature(help, "format", 1);

        bridge.shutdown().await.unwrap();
    });
}

#[test]
fn signature_help_tracks_imported_callable_edits() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let fixture = Fixture::new(&corsa_path);
        let dependency_path = fixture.root.path().join("src/format.ts");
        std::fs::write(
            &dependency_path,
            "export declare function format(value: string, precision: number): string;\n",
        )
        .unwrap();
        let source = "<script setup lang=\"ts\">\nimport { format } from './format';\n</script>\n<template>{{ format('imported', ) }}</template>\n";
        let (state, uri) = fixture.vue("Imported.vue", source);
        let bridge = fixture.bridge();
        bridge.spawn().await.unwrap();

        let marker = "format('imported', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let help = SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("signature help from imported callable");
        assert_signature_type(&help, "precision: number");

        let updated = "export declare function format(value: string, precision: bigint): string;\n";
        std::fs::write(&dependency_path, updated).unwrap();
        bridge
            .open_or_update_virtual_document(dependency_path.to_str().unwrap(), updated)
            .await
            .unwrap();

        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let help = SignatureHelpService::signature_help_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("signature help after imported callable edit");
        assert_signature_type(&help, "precision: bigint");

        bridge.shutdown().await.unwrap();
    });
}

fn assert_signature(help: tower_lsp::lsp_types::SignatureHelp, name: &str, active_parameter: u32) {
    assert_eq!(help.active_signature, Some(0));
    assert_eq!(help.active_parameter, Some(active_parameter));
    assert_eq!(help.signatures.len(), 1);
    let signature = &help.signatures[0];
    assert!(signature.label.contains(name), "{}", signature.label);
    assert!(
        signature.label.contains("value: string"),
        "{}",
        signature.label
    );
    assert!(
        signature.label.contains("precision: number"),
        "{}",
        signature.label
    );
    assert_eq!(
        signature.parameters.as_ref().map(Vec::len),
        Some(2),
        "{}",
        signature.label
    );
}

fn assert_signature_type(help: &tower_lsp::lsp_types::SignatureHelp, expected: &str) {
    assert_eq!(help.signatures.len(), 1);
    assert!(
        help.signatures[0].label.contains(expected),
        "{}",
        help.signatures[0].label
    );
}
