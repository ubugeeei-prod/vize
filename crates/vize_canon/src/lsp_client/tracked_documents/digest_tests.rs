use super::*;

fn digest(text: &str) -> [u8; 32] {
    Sha256::digest(text.as_bytes()).into()
}

#[test]
fn a_thousand_retained_inputs_hash_once_and_one_edit_hashes_only_its_new_revision() {
    let inputs = (0..1000)
        .map(|index| {
            (
                String::from(format!("file:///workspace/Member{index}.vue.ts")),
                String::from(format!("// 雪😀\r\nexport const value = {index};\r\n")),
            )
        })
        .collect::<Vec<_>>();
    let mut documents = TrackedDocumentTexts::default();
    for (uri, text) in &inputs {
        documents.insert(uri.clone(), text.clone());
    }
    for _ in 0..5 {
        for (uri, text) in &inputs {
            assert_eq!(documents.desired_digest(uri, text), digest(text));
        }
        assert_eq!(documents.digest_computations, inputs.len());
    }
    let (uri, _) = &inputs[777];
    let edited = "// 雪😀\r\nexport const value = 'unsaved';\r\n";
    documents.insert(uri.clone(), edited.into());
    for (input_uri, original) in &inputs {
        let expected = if input_uri == uri { edited } else { original };
        assert_eq!(
            documents.desired_digest(input_uri, expected),
            digest(expected)
        );
    }
    assert_eq!(documents.digest_computations, inputs.len() + 1);
}

#[test]
fn earlier_duplicate_rows_and_uri_aliases_never_borrow_a_different_desired_digest() {
    let mut documents = TrackedDocumentTexts::default();
    let uri = "file:///workspace/A.vue.ts";
    let alias = "file:///workspace/%41.vue.ts";
    let before = "export const value = 1;\n";
    let after = "export const value = 2;\n";
    documents.insert(uri.into(), before.into());
    documents.insert(alias.into(), after.into());
    assert_eq!(documents.desired_digest(uri, before), digest(before));
    assert_eq!(documents.desired_digest(alias, after), digest(after));
    documents.insert(uri.into(), after.into());
    // Retaining an entire batch leaves its final row here. Replaying the
    // earlier row must still observe that earlier row's complete bytes.
    assert_eq!(documents.desired_digest(uri, before), digest(before));
    assert_eq!(documents.desired_digest(uri, after), digest(after));
    assert_eq!(documents.desired_digest(alias, before), digest(before));
    assert_eq!(documents.desired_digest(alias, after), digest(after));
    assert_eq!(
        documents.desired_digest("file:///absent.ts", before),
        digest(before)
    );
}

#[test]
fn desired_observations_follow_close_reopen_clone_replay_and_clear_ownership() {
    let uri = "file:///workspace/A.vue.ts";
    let before = "export const value = '雪';\r\n";
    let after = "export const value = '😀';\r\n";
    let mut documents = TrackedDocumentTexts::default();
    documents.insert(uri.into(), before.into());
    assert_eq!(documents.desired_digest(uri, before), digest(before));
    documents.insert_if_absent(uri.into(), after.into());
    assert_eq!(documents.desired_digest(uri, before), digest(before));
    assert_eq!(documents.digest_computations, 1);
    let mut cloned = documents.clone();
    cloned.insert(uri.into(), after.into());
    assert_eq!(cloned.desired_digest(uri, after), digest(after));
    assert_eq!(documents.desired_digest(uri, before), digest(before));
    let retained = std::mem::take(&mut documents);
    for (uri, text) in retained {
        documents.insert_if_absent(uri, text);
    }
    assert_eq!(documents.desired_digest(uri, before), digest(before));
    documents.remove(uri);
    documents.insert(uri.into(), after.into());
    assert_eq!(documents.desired_digest(uri, after), digest(after));
    documents.clear();
    assert!(documents.desired_digests.is_empty());
    documents.insert(uri.into(), before.into());
    assert_eq!(documents.desired_digest(uri, before), digest(before));
}
