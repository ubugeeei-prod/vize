use super::*;
use crate::virtual_code::TemplateOperandProducts;

fn store(source: &str) -> (DocumentStore, SourceSnapshotCache, Url) {
    let (documents, cache, uri) = open();
    assert!(documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: source.into(),
        }],
        2
    ));
    (documents, cache, uri)
}

fn assert_script(source: &str, uri: &Url, products: &TemplateOperandProducts) {
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let expected = VirtualCodeGenerator::new()
        .generate(&descriptor, uri.as_str())
        .script_setup
        .unwrap();
    let actual = products.script_setup.as_ref().unwrap();
    assert_eq!(
        (
            &actual.uri,
            &actual.content,
            actual.language,
            &actual.source_map
        ),
        (
            &expected.uri,
            &expected.content,
            expected.language,
            &expected.source_map
        )
    );
}

#[test]
fn explicit_listener_request_reuses_original_named_event_and_shorthand_products() {
    let source = "<script setup>import { save as 保存 } from './actions'; const title='送信';</script>\r\n<template><button :title @click.once='保存'>{{ title }}</button></template>";
    let (documents, cache, uri) = store(source);
    let (base, _) = cache.begin_query(&documents, &uri).unwrap();
    assert!(matches!(
        block_on(base.run_native_template_products(&documents, options())),
        Err(NativeTemplateQueryError::File(_))
    ));
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let result =
        block_on(query.run_native_listener_template_products(&documents, options())).unwrap();
    let products = result.publish(&documents, |products| products).unwrap();
    assert_eq!(products.checker.as_str(), "title\n保存\ntitle");
    assert_eq!(products.checker.links().len(), 3);
    assert_script(source, &uri, &products);
}

#[cfg(feature = "experimental-lexical-arrow-queries")]
#[test]
fn explicit_lexical_request_keeps_arrow_captures_and_public_script_exports() {
    let source = "<script setup lang='ts'>const 外=2; const save=(n:number)=>n+外;</script>\r\n<template><button @click.once='save'/></template>";
    let (documents, cache, uri) = store(source);
    let (listener, _) = cache.begin_query(&documents, &uri).unwrap();
    assert!(matches!(
        block_on(listener.run_native_listener_template_products(&documents, options())),
        Err(NativeTemplateQueryError::File(_))
    ));
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let result =
        block_on(query.run_native_lexical_template_products(&documents, options())).unwrap();
    let products = result.publish(&documents, |products| products).unwrap();
    assert_eq!(products.checker.as_str(), "save");
    assert_script(source, &uri, &products);
}

#[test]
fn listener_products_cannot_publish_after_the_original_revision_changes() {
    let source = "<script setup>import {save} from './actions';</script><template><button @click='save'/></template>";
    let (documents, cache, uri) = store(source);
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let result =
        block_on(query.run_native_listener_template_products(&documents, options())).unwrap();
    assert!(documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: source.into()
        }],
        3
    ));
    let published = AtomicBool::new(false);
    assert!(matches!(
        result.publish(&documents, |_| published.store(true, Ordering::Relaxed)),
        Err(SnapshotRefusal::Superseded)
    ));
    assert!(!published.load(Ordering::Relaxed));
}

#[test]
fn explicit_listener_request_keeps_dynamic_inline_and_script_call_refusals() {
    for source in [
        "<script setup>import {save} from './actions'; const event='click';</script><template><button @[event]='save'/></template>",
        "<script setup>import {save} from './actions';</script><template><button @click='save()'/></template>",
        "<script setup>import {save} from './actions'; const result=save();</script><template><button @click='save'/></template>",
    ] {
        let (documents, cache, uri) = store(source);
        let (query, _) = cache.begin_query(&documents, &uri).unwrap();
        assert!(
            matches!(
                block_on(query.run_native_listener_template_products(&documents, options())),
                Err(NativeTemplateQueryError::File(_))
            ),
            "{source}"
        );
    }
}
