use std::time::Duration;

use corsa_lsp::{LspClient, LspOverlay};
use lsp_types::Uri;
use serde_json::Value;

use super::{
    assert_completion, assert_no_generated_uri, assert_no_generated_uri_or_zero_range,
    contains_location, definition, hover, position,
};

pub async fn assert_component_completions(
    client: &LspClient,
    overlay: &LspOverlay,
    uri: &Uri,
    source: &str,
) {
    let partial = source
        .replace(":count", ":cou")
        .replace("@save=", "@sa=")
        .replace("@save-item", "@save-");
    let prop_position = position(&partial, partial.find(":cou").unwrap() + ":cou".len());
    let event_position = position(&partial, partial.find("@sa").unwrap() + "@sa".len());
    let kebab_event_position = position(&partial, partial.find("@save-").unwrap() + "@save-".len());
    overlay.replace(uri, partial.as_str()).unwrap();
    let uri_text = uri.as_str();
    assert_completion(
        client,
        uri_text,
        &prop_position,
        "count",
        &["(property) count: number"],
    )
    .await;
    assert_completion(client, uri_text, &event_position, "save", &["number"]).await;
    assert_completion(
        client,
        uri_text,
        &kebab_event_position,
        "save-item",
        &["string"],
    )
    .await;
    overlay.replace(uri, source).unwrap();
}

pub async fn assert_component_navigation(
    client: &LspClient,
    uri: &str,
    position: &Value,
    component_name: &str,
    target_uri: &str,
) {
    let component_hover = hover(client, uri, position).await;
    let hover_text = serde_json::to_string(&component_hover).unwrap();
    let component_hover_matches =
        hover_text.contains(component_name) && !hover_text.contains("__vize_component__");
    assert!(component_hover_matches, "{component_hover:#}");
    let component_definition = definition(client, uri, position).await;
    assert_no_generated_uri(&component_definition);
    let definition_text = serde_json::to_string(&component_definition).unwrap();
    let definition_targets_authored_file = definition_text.contains(target_uri);
    let definition_hides_virtual_file = !definition_text.contains(".vue.ts");
    assert!(definition_targets_authored_file, "{component_definition:#}");
    assert!(definition_hides_virtual_file, "{component_definition:#}");
}

pub async fn assert_prop_navigation(
    client: &LspClient,
    uri: &str,
    position: &Value,
    prop_name: &str,
    prop_type: &str,
    target_uri: &str,
    target_start: &Value,
) {
    let mut prop_hover = hover(client, uri, position).await;
    let mut hover_text = serde_json::to_string(&prop_hover).unwrap();
    let mut prop_definition = definition(client, uri, position).await;
    for _ in 0..60 {
        let hover_ready = hover_text.contains(prop_name) && hover_text.contains(prop_type);
        let definition_ready = contains_location(&prop_definition, target_uri, target_start);
        if hover_ready && definition_ready {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
        if !hover_ready {
            prop_hover = hover(client, uri, position).await;
            hover_text = serde_json::to_string(&prop_hover).unwrap();
        }
        if !definition_ready {
            prop_definition = definition(client, uri, position).await;
        }
    }
    let hover_ready = hover_text.contains(prop_name) && hover_text.contains(prop_type);
    assert!(
        hover_ready,
        "{prop_name} hover should include {prop_type}:\nhover:\n{prop_hover:#}\ndefinition:\n{prop_definition:#}"
    );
    assert_no_generated_uri_or_zero_range(&prop_definition);
    assert!(
        contains_location(&prop_definition, target_uri, target_start),
        "{prop_name} definition should map to authored declaration:\n{prop_definition:#}"
    );
}

pub async fn assert_component_members(
    client: &LspClient,
    parent: (&str, &str),
    child: (&str, &str),
) {
    for (usage, declaration, name, ty) in [
        (":count", "count: number", "count", "number"),
        ("@save", "save: [value", "save", "number"),
    ] {
        let usage = position(parent.1, parent.1.find(usage).unwrap() + 1);
        let declaration = position(child.1, child.1.find(declaration).unwrap());
        assert_prop_navigation(client, parent.0, &usage, name, ty, child.0, &declaration).await;
    }
}
