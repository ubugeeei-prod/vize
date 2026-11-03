//! The observable view: rendered trees, their JSON form and the interaction
//! targets of a materialized tree.

use super::incremental;
use super::json::Json;

/// What an element shows besides its address, tag and children.
#[derive(Clone, Debug, PartialEq)]
pub struct Payload {
    pub attributes: Vec<(String, Json)>,
    pub disabled: bool,
    pub event: Option<Json>,
}

pub type View = incremental::View<String, Payload>;
pub type Node = incremental::Node<String, Payload>;
pub type State = incremental::State<String, Payload>;

pub fn element(
    address: String,
    tag: String,
    attributes: Vec<(String, Json)>,
    disabled: bool,
    event: Option<Json>,
    children: Vec<View>,
) -> View {
    View::Element {
        address,
        tag,
        payload: Payload {
            attributes,
            disabled,
            event,
        },
        children,
    }
}

/// An object from ordered pairs; a later duplicate key wins.
pub fn object(pairs: &[(String, Json)]) -> Json {
    Json::Object(pairs.iter().cloned().collect())
}

pub fn json(view: &View) -> Json {
    match view {
        View::Text(value) => Json::String(value.clone()),
        View::Element {
            tag,
            payload,
            children,
            ..
        } => {
            let mut fields = vec![
                ("tag".to_owned(), Json::String(tag.clone())),
                ("attributes".to_owned(), object(&payload.attributes)),
                (
                    "children".to_owned(),
                    Json::Array(children.iter().map(json).collect()),
                ),
            ];
            if tag == "button" {
                fields.push(("disabled".to_owned(), Json::Bool(payload.disabled)));
            }
            object(&fields)
        }
    }
}

/// Append in document order, merging adjacent text and dropping empty text.
pub fn append(nodes: &mut Vec<View>, node: View) {
    match (nodes.last_mut(), node) {
        (_, View::Text(value)) if value.is_empty() => {}
        (Some(View::Text(previous)), View::Text(value)) => previous.push_str(&value),
        (_, node) => nodes.push(node),
    }
}

pub fn tree(nodes: &[View]) -> Json {
    Json::Array(nodes.iter().map(json).collect())
}

pub fn state_tree(state: &State) -> Json {
    tree(&incremental::erase_all(&state.roots))
}

/// An element carrying a `data-id` string attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub name: String,
    pub identity: u32,
    pub tag: String,
    pub disabled: bool,
    pub event: Option<Json>,
}

pub fn targets(node: &Node, out: &mut Vec<Target>) {
    let Node::Element {
        identity,
        tag,
        payload,
        children,
        ..
    } = node
    else {
        return;
    };
    if let Some((_, Json::String(name))) = payload.attributes.iter().find(|(k, _)| k == "data-id") {
        out.push(Target {
            name: name.clone(),
            identity: *identity,
            tag: tag.clone(),
            disabled: payload.disabled,
            event: payload.event.clone(),
        });
    }
    for child in children {
        targets(child, out);
    }
}

pub fn all_targets(state: &State) -> Vec<Target> {
    let mut out = Vec::new();
    for root in &state.roots {
        targets(root, &mut out);
    }
    out
}

pub fn identities(state: &State) -> Json {
    Json::Array(
        all_targets(state)
            .into_iter()
            .map(|t| Json::Array(vec![Json::String(t.name), Json::from(t.identity)]))
            .collect(),
    )
}

/// The event a click on the one live target `name` delivers.
pub fn click(state: &State, name: &str) -> Result<Option<Json>, String> {
    let found: Vec<Target> = all_targets(state)
        .into_iter()
        .filter(|t| t.name == name)
        .collect();
    let [target] = found.as_slice() else {
        return Err("expected one live interaction target".into());
    };
    if target.tag != "button" {
        return Err("expected a native button".into());
    }
    Ok(if target.disabled {
        None
    } else {
        target.event.clone()
    })
}
