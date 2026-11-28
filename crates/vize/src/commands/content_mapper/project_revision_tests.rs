//! The project revision a transform belongs to is the last `openProject` the
//! host sent for its handle before the transform, and nothing else.
//!
//! The host owns files and overlays: an edit, save, create, rename or delete
//! reaches the mapper only as the next `transform` for a file name and its
//! content. So one host sequence across every lifecycle step must give each
//! transform the reply that a fresh server returns for that transform's
//! revision alone. No earlier overlay, file name or settings snapshot may leak
//! into it, a close must end the revision, and a reopen must start a new one.
//! Replies are checked in request order and by id in ready order, because
//! the editor host pairs replies by id.

use std::io::{BufReader, Cursor};

use serde_json::{Value, json};
use vize_l0::cstr;

use super::dispatch::ReplyOrder;
use super::test_support::{
    close_project_request, frames, initialize_request, open_project_request,
};

const OPTIONS_V1: &str = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;

const OPTIONS_V2: &str = r#"<script lang="ts">
export default { data() { return { count: 1, label: 'n' } } }
</script>
<template>{{ count }} {{ label }}</template>
"#;

const SETUP: &str = r#"<script setup lang="ts">
const total: number = 1
</script>
<template>{{ total }}</template>
"#;

fn open(id: u8, options_api: bool) -> Value {
    open_project_request(id, json!({ "optionsApi": options_api }), json!({}))
}

fn transform(id: u8, file: &str, content: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "transform",
        "params": { "fileName": file, "content": content, "projectHandle": "p1" }
    })
}

fn serve(requests: &[Value], order: ReplyOrder) -> Vec<Value> {
    let input = frames(requests);
    let mut reader = BufReader::new(Cursor::new(input));
    let mut output = Vec::new();
    super::dispatch::serve(&mut reader, &mut output, false, false, order).unwrap();
    let mut reader = BufReader::new(Cursor::new(output));
    let mut replies = Vec::new();
    while let Some(body) = super::read_frame(&mut reader).unwrap() {
        replies.push(serde_json::from_slice(&body).unwrap());
    }
    replies
}

/// The reply a fresh server gives `request` under the revision `opened`.
fn fresh(opened: &Value, request: &Value) -> Value {
    let mut request = request.clone();
    request["id"] = json!(3);
    let mut opened = opened.clone();
    opened["id"] = json!(2);
    let mut replies = serve(
        &[initialize_request(), opened, request],
        ReplyOrder::Request,
    );
    assert_eq!(replies.len(), 3);
    replies.pop().unwrap()
}

/// One host session: every lifecycle step against one project handle.
fn session() -> Vec<Value> {
    vec![
        initialize_request(),
        open(2, true),
        // open, edit and save one file.
        transform(3, "/project/A.vue", OPTIONS_V1),
        transform(4, "/project/A.vue", OPTIONS_V2),
        transform(5, "/project/A.vue", OPTIONS_V2),
        // The project's options change: a new revision of the same handle.
        open(6, false),
        transform(7, "/project/A.vue", OPTIONS_V2),
        // create, then rename the created file.
        transform(8, "/project/B.vue", SETUP),
        transform(9, "/project/C.vue", SETUP),
        // close ends the revision; the deleted name is never asked again.
        close_project_request(10),
        transform(11, "/project/C.vue", SETUP),
        // A reopen starts a new revision with the first options.
        open(12, true),
        transform(13, "/project/A.vue", OPTIONS_V1),
        transform(14, "/project/C.vue", SETUP),
    ]
}

fn by_id(replies: Vec<Value>) -> Vec<(u64, Value)> {
    let mut replies = replies
        .into_iter()
        .map(|reply| (reply["id"].as_u64().unwrap(), reply))
        .collect::<Vec<_>>();
    replies.sort_by_key(|(id, _)| *id);
    replies
}

#[test]
fn every_transform_belongs_to_the_last_revision_opened_before_it() {
    let requests = session();
    for order in [ReplyOrder::Request, ReplyOrder::Ready] {
        let replies = serve(&requests, order);
        let ids = replies
            .iter()
            .map(|reply| reply["id"].as_u64().unwrap())
            .collect::<Vec<_>>();
        if order == ReplyOrder::Request {
            assert_eq!(ids, (1..=14).collect::<Vec<_>>());
        }
        let replies = by_id(replies);
        assert_eq!(
            replies.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            (1..=14).collect::<Vec<_>>(),
            "{order:?}: every request has exactly one reply"
        );

        let mut revision = None::<&Value>;
        for (request, (id, reply)) in requests.iter().zip(&replies) {
            let label = cstr!("{order:?} request {id}");
            match request["method"].as_str().unwrap() {
                "openProject" => {
                    assert!(reply.get("error").is_none(), "{label}: {reply}");
                    revision = Some(request);
                }
                "closeProject" => {
                    assert_eq!(reply["result"], Value::Null, "{label}");
                    revision = None;
                }
                "transform" => match revision {
                    Some(opened) => {
                        assert!(reply.get("error").is_none(), "{label}: {reply}");
                        assert_eq!(reply["result"], fresh(opened, request)["result"], "{label}");
                    }
                    None => assert_eq!(reply["error"]["code"], -32602, "{label}: {reply}"),
                },
                _ => {}
            }
        }

        let result = |id: usize| &replies[id - 1].1["result"];
        // A save without changes keeps the edit's reply byte for byte.
        assert_eq!(result(5), result(4), "{order:?}");
        // The options change reaches the next transform of the same overlay.
        assert_ne!(result(7), result(5), "{order:?}");
        // The reopened revision drops the closed one's options.
        assert_eq!(result(13), result(3), "{order:?}");
    }
}
