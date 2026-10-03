use vize_l0::Allocator;
use vize_l1_to_l2::native_file::lower_sfc_native;

use super::*;

#[test]
fn explicit_cancellation_before_first_poll_refuses_the_real_sfc_query() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///cancelled-native-comments.vue").unwrap();
    project.open(
        uri.clone(),
        "<script>/*\n never published\n*/ const value = 1;</script>".into(),
        1,
        "vue".into(),
    );
    let (query, cancel) = project.begin_query(&uri).unwrap();
    cancel.abort();
    assert!(matches!(
        block_on(query_sfc_comments(query, options())),
        Err(SnapshotRefusal::Cancelled)
    ));
}

#[test]
fn explicit_cancellation_after_native_computation_prevents_publication() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///ready-native-comments.vue").unwrap();
    project.open(
        uri.clone(),
        "<script>/*\n ready\n*/ const value = 1;</script>".into(),
        1,
        "vue".into(),
    );
    let (query, cancel) = project.begin_query(&uri).unwrap();
    let ready = block_on(query_sfc_comments(query, options())).unwrap();
    cancel.abort();
    let mut published = false;
    assert_eq!(
        ready.publish(|_| published = true),
        Err(SnapshotRefusal::Cancelled)
    );
    assert!(!published);
}

#[test]
fn rejected_template_and_recovery_return_original_typed_issues_not_empty_success() {
    for source in [
        "<script>/*\n retained\n*/ const value = 1;</script><template><div v-if=\"value\" /></template>",
        "<script>/*\n retained\n*/ const = ;</script>",
    ] {
        let documents = DocumentStore::new();
        let project = SourceQueryProject::new(&documents);
        let uri = Url::parse("file:///refused-native-comments.vue").unwrap();
        project.open(uri.clone(), source.into(), 1, "vue".into());
        let (query, _) = project.begin_query(&uri).unwrap();
        let original = query.snapshot().clone();
        let arena = Allocator::default();
        let observed = lower_sfc_native(&arena, original.source(), options());
        assert!(core::ptr::eq(
            observed.descriptor().source(),
            original.source()
        ));
        assert!(observed.scripts()[0].syntax().is_some());
        assert!(observed.admitted().is_none());
        let expected_issues = observed.issues().to_vec();
        assert!(!expected_issues.is_empty());
        let ready = block_on(query_sfc_comments(query, options())).unwrap();
        assert_eq!(
            ready.publish(|value| value),
            Ok(Err(SfcQueryRefusal::Producer(expected_issues)))
        );
    }
}

#[test]
fn unsupported_union_annotation_keeps_real_syntax_but_refuses_native_file_projection() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///typed-native-comment-refusal.vue").unwrap();
    // The original union type parses normally but remains outside the genuine
    // File family; a host cannot promote its comments into an admitted File.
    let source = "\r\n<script lang=\"ts\">\r\n/* 😀\u{2028}\u{2029}\r\n 日本語\r\n*/\r\nconst value: number | string = 1;\r\n</script>";
    project.open(uri.clone(), source.into(), 7, "vue".into());
    let (query, _) = project.begin_query(&uri).unwrap();
    let retained = query.snapshot().clone();
    let arena = Allocator::default();
    let observed = lower_sfc_native(&arena, retained.source(), options());
    let syntax = observed.scripts()[0].syntax().unwrap();
    assert!(syntax.admitted_program().is_some());
    assert!(syntax.source_type().is_typescript());
    assert!(core::ptr::eq(
        syntax.source().text(),
        observed.scripts()[0].block().source()
    ));
    assert!(observed.admitted().is_none());
    let original_issues = observed.issues().to_vec();
    assert!(!original_issues.is_empty());
    let ready = block_on(query_sfc_comments(query, options())).unwrap();
    assert_eq!(
        ready.publish(|value| value),
        Ok(Err(SfcQueryRefusal::Producer(original_issues)))
    );
}

#[test]
fn primitive_keyword_annotation_keeps_real_snapshot_and_full_sfc_comment_projection() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///typed-native-comment-admission.vue").unwrap();
    let source = "\r\n<script lang=\"ts\">\r\n/* 😀\u{2028}\u{2029}\r\n 日本語\r\n*/\r\nconst value: number = 1;\r\n</script>";
    project.open(uri.clone(), source.into(), 7, "vue".into());
    let (query, _) = project.begin_query(&uri).unwrap();
    let retained = query.snapshot().clone();
    assert_eq!(retained.source(), source);
    assert_eq!(retained.uri(), &uri);
    assert_eq!(retained.version(), 7);
    let arena = Allocator::default();
    let observed = lower_sfc_native(&arena, retained.source(), options());
    let native = observed
        .admitted()
        .expect("actual complete original keyword File");
    assert!(core::ptr::eq(
        native.file().file().artifact().source(),
        retained.source()
    ));
    let ready = block_on(query_sfc_comments(query, options())).unwrap();
    assert_eq!(ready.publish(|value| value), Ok(Ok(vec![expected(2, 3)])));
    assert_eq!(
        retained.native_file(&documents),
        Err(SnapshotRefusal::NativeFileUnavailable)
    );
}
