//! Restriction lists whose entries carry more than a name.
//!
//! `vue/no-restricted-html-elements` and `vue/no-restricted-v-on` accept a
//! name as before. Each also accepts eslint-plugin-vue's object entry, which
//! narrows the restriction and can replace the default message.

use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned, de::Error};

use super::RestrictedNames;
use crate::String;

/// Read one entry: a name, or an object as `T`. Only a JSON object is the
/// object form; a derived struct would also read a list positionally.
fn entry<'de, D, T, E>(
    deserializer: D,
    name: fn(String) -> E,
    spec: fn(T) -> E,
) -> Result<E, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_object() {
        T::deserialize(value).map(spec).map_err(D::Error::custom)
    } else {
        String::deserialize(value)
            .map(name)
            .map_err(D::Error::custom)
    }
}

/// `vue/no-restricted-html-elements`: one name, or a list of entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RestrictedHtmlElements {
    /// One element name.
    One(String),
    /// Names and `{ element, message }` entries, in authored order.
    Many(Vec<RestrictedHtmlElement>),
}

/// One `vue/no-restricted-html-elements` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum RestrictedHtmlElement {
    /// An element name reported with the default message.
    Name(String),
    /// Element names that share a message.
    Spec(RestrictedHtmlElementSpec),
}

impl<'de> Deserialize<'de> for RestrictedHtmlElement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        entry(deserializer, Self::Name, Self::Spec)
    }
}

/// Element names with the message reported for them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedHtmlElementSpec {
    /// One element name or several.
    pub element: RestrictedNames,
    /// Message that replaces the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl RestrictedHtmlElements {
    /// Every restricted name with its entry's message, in authored order.
    /// The first entry that names an element decides its message.
    pub fn restrictions(&self) -> Vec<(String, Option<String>)> {
        let entries = match self {
            Self::One(name) => return vec![(name.clone(), None)],
            Self::Many(entries) => entries,
        };
        let mut restrictions = Vec::with_capacity(entries.len());
        for entry in entries {
            match entry {
                RestrictedHtmlElement::Name(name) => restrictions.push((name.clone(), None)),
                RestrictedHtmlElement::Spec(spec) => {
                    for name in spec.element.names() {
                        restrictions.push((name, spec.message.clone()));
                    }
                }
            }
        }
        restrictions
    }
}

/// `vue/no-restricted-v-on`: one event name, or a list of entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RestrictedVOn {
    /// One event name.
    One(String),
    /// Names and `{ argument, modifiers, element, message }` entries.
    Many(Vec<RestrictedVOnEntry>),
}

/// One `vue/no-restricted-v-on` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum RestrictedVOnEntry {
    /// An event name restricted on every element.
    Name(String),
    /// An event restricted with modifiers, on one element, or with a message.
    Spec(RestrictedVOnSpec),
}

impl<'de> Deserialize<'de> for RestrictedVOnEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        entry(deserializer, Self::Name, Self::Spec)
    }
}

/// One restricted listener.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedVOnSpec {
    /// Event name, the `v-on` argument.
    pub argument: String,
    /// Modifiers the listener must all carry. Empty restricts every listener.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
    /// Authored tag name the restriction is limited to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    /// Message that replaces the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl RestrictedVOn {
    /// Every configured restriction, in authored order. A bare name restricts
    /// the event everywhere.
    pub fn restrictions(&self) -> Vec<RestrictedVOnSpec> {
        let named = |argument: &String| RestrictedVOnSpec {
            argument: argument.clone(),
            modifiers: Vec::new(),
            element: None,
            message: None,
        };
        match self {
            Self::One(name) => vec![named(name)],
            Self::Many(entries) => entries
                .iter()
                .map(|entry| match entry {
                    RestrictedVOnEntry::Name(name) => named(name),
                    RestrictedVOnEntry::Spec(spec) => spec.clone(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::ConfigLintRuleOptions;
    use super::{RestrictedVOnSpec, String};

    fn parse(json: &str) -> Result<ConfigLintRuleOptions, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn elements(json: &str) -> Vec<(String, Option<String>)> {
        parse(json)
            .unwrap()
            .configured_vue()
            .no_restricted_html_elements
            .as_ref()
            .unwrap()
            .restrictions()
    }

    #[test]
    fn element_names_and_messaged_entries_flatten_in_authored_order() {
        let name = |name: &str| (String::from(name), None);
        let messaged = |name: &str, message: &str| (String::from(name), Some(message.into()));
        assert_eq!(
            elements(r#"{"vue/no-restricted-html-elements":"marquee"}"#),
            [name("marquee")]
        );
        assert_eq!(
            elements(r#"{"vue/no-restricted-html-elements":["marquee","blink"]}"#),
            [name("marquee"), name("blink")]
        );
        assert_eq!(
            elements(
                r#"{"vue/no-restricted-html-elements":[
                    "marquee",
                    {"element":["a","RouterLink"],"message":"Use Bar."},
                    {"element":"button","message":"Use BaseButton."},
                    {"element":"blink"}
                ]}"#
            ),
            [
                name("marquee"),
                messaged("a", "Use Bar."),
                messaged("RouterLink", "Use Bar."),
                messaged("button", "Use BaseButton."),
                name("blink"),
            ]
        );
        assert_eq!(
            elements(r#"{"vue/no-restricted-html-elements":[]}"#),
            Vec::new()
        );
    }

    #[test]
    fn element_entries_outside_the_documented_shape_are_refused() {
        for json in [
            r#"{"vue/no-restricted-html-elements":{"element":"a"}}"#,
            r#"{"vue/no-restricted-html-elements":[{"message":"Use Bar."}]}"#,
            r#"{"vue/no-restricted-html-elements":[{"element":"a","msg":"Use Bar."}]}"#,
            r#"{"vue/no-restricted-html-elements":[{"element":1}]}"#,
            r#"{"vue/no-restricted-html-elements":[{"element":"a","message":1}]}"#,
            r#"{"vue/no-restricted-html-elements":[["a"]]}"#,
        ] {
            assert!(parse(json).is_err(), "accepted invalid option: {json}");
        }
    }

    #[test]
    fn listener_names_and_object_entries_become_restrictions() {
        let listeners = |json: &str| {
            parse(json)
                .unwrap()
                .configured_vue()
                .no_restricted_v_on
                .as_ref()
                .unwrap()
                .restrictions()
        };
        let named = |argument: &str| RestrictedVOnSpec {
            argument: argument.into(),
            modifiers: Vec::new(),
            element: None,
            message: None,
        };
        assert_eq!(
            listeners(r#"{"vue/no-restricted-v-on":"click"}"#),
            [named("click")]
        );
        assert_eq!(
            listeners(
                r#"{"vue/no-restricted-v-on":[
                    "legacy-click",
                    {"argument":"click","modifiers":["stop"]},
                    {"argument":"focus","element":"MyInput","message":"Use focusin."}
                ]}"#
            ),
            [
                named("legacy-click"),
                RestrictedVOnSpec {
                    modifiers: vec!["stop".into()],
                    ..named("click")
                },
                RestrictedVOnSpec {
                    element: Some("MyInput".into()),
                    message: Some("Use focusin.".into()),
                    ..named("focus")
                },
            ]
        );
    }

    #[test]
    fn listener_entries_outside_the_documented_shape_are_refused() {
        for json in [
            r#"{"vue/no-restricted-v-on":{"argument":"click"}}"#,
            r#"{"vue/no-restricted-v-on":[{"modifiers":["stop"]}]}"#,
            r#"{"vue/no-restricted-v-on":[{"argument":null}]}"#,
            r#"{"vue/no-restricted-v-on":[null]}"#,
            r#"{"vue/no-restricted-v-on":[{"argument":"click","modifiers":"stop"}]}"#,
            r#"{"vue/no-restricted-v-on":[{"argument":"click","modifier":["stop"]}]}"#,
            r#"{"vue/no-restricted-v-on":[["click"]]}"#,
        ] {
            assert!(parse(json).is_err(), "accepted invalid option: {json}");
        }
    }

    #[test]
    fn a_later_layer_replaces_each_list_whole() {
        let mut options = parse(
            r#"{
                "vue/no-restricted-html-elements":[{"element":"a","message":"Use Bar."}],
                "vue/no-restricted-v-on":[{"argument":"click","modifiers":["stop"]}]
            }"#,
        )
        .unwrap();
        options.merge_from(&parse(r#"{"vue/no-restricted-html-elements":"marquee"}"#).unwrap());
        let configured = options.configured_vue();
        assert_eq!(
            configured
                .no_restricted_html_elements
                .as_ref()
                .unwrap()
                .restrictions(),
            [(String::from("marquee"), None)]
        );
        assert_eq!(
            configured
                .no_restricted_v_on
                .as_ref()
                .unwrap()
                .restrictions()
                .len(),
            1
        );
    }
}
