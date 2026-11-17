//! `type/unbound-method` trusts platform methods of the global object.
//!
//! `window.setTimeout`, `{ matchMedia } = window` and
//! `self.requestAnimationFrame` read a WebIDL operation off the global object.
//! Called without a receiver, such an operation uses the global object, so
//! nothing is lost. typescript-eslint reports them because `Window` declares
//! them as methods; `checkGlobalObjectMethods: true` restores that.
use corsa::lint::LintNode;
use serde_json::{Value, json};

/// Member-fact fields the rule reads, on the node or its `object` child.
const MEMBER_FACTS: [&str; 2] = ["__memberMethods", "__initMemberMethods"];
/// The single-method fact of a computed access.
const METHOD_FACT: &str = "__unboundMethodInfo";

pub(super) fn trust_global_object(node: &mut LintNode, configured: Option<&Value>) {
    if checks_global_object_methods(configured) {
        return;
    }
    trust_facts(node);
    if let Some(object) = node.children.get_mut("object") {
        trust_facts(object);
    }
    if let Some(properties) = node.child_lists.get_mut("properties") {
        properties.iter_mut().for_each(trust_facts);
    }
}

fn checks_global_object_methods(configured: Option<&Value>) -> bool {
    configured
        .and_then(|options| options.get(0))
        .and_then(|options| options.get("checkGlobalObjectMethods"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Present each global-object library member as a non-method.
fn trust_facts(node: &mut LintNode) {
    for key in MEMBER_FACTS {
        let Some(members) = node
            .fields
            .get_mut(key)
            .and_then(|facts| facts.get_mut("members"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        for member in members.values_mut() {
            if is_global_object_member(member)
                && let Some(member) = member.as_object_mut()
            {
                member.insert("parts".into(), json!([{ "isMethod": false }]));
            }
        }
    }
    if let Some(method) = node.fields.get_mut(METHOD_FACT)
        && is_global_object_member(method)
    {
        *method = json!({ "isMethod": false });
    }
}

fn is_global_object_member(fact: &Value) -> bool {
    fact.get("globalObjectMember").and_then(Value::as_bool) == Some(true)
}

#[cfg(test)]
mod tests {
    use super::trust_global_object;
    use corsa::lint::LintNode;
    use serde_json::json;

    fn member(global: bool) -> LintNode {
        serde_json::from_value(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 17 },
            "children": { "object": {
                "kind": "Identifier",
                "range": { "start": 0, "end": 6 },
                "fields": { "__memberMethods": { "members": { "setTimeout": {
                    "parts": [{ "isMethod": true }],
                    "propertyLib": true,
                    "globalObjectMember": global
                } } } }
            } }
        }))
        .expect("member node")
    }

    fn parts(node: &LintNode) -> serde_json::Value {
        node.children["object"].fields["__memberMethods"]["members"]["setTimeout"]["parts"].clone()
    }

    #[test]
    fn global_object_members_are_trusted_unless_configured() {
        let mut trusted = member(true);
        trust_global_object(&mut trusted, None);
        assert_eq!(parts(&trusted), json!([{ "isMethod": false }]));

        let mut checked = member(true);
        let options = json!([{ "ignoreStatic": false, "checkGlobalObjectMethods": true }]);
        trust_global_object(&mut checked, Some(&options));
        assert_eq!(parts(&checked), json!([{ "isMethod": true }]));

        let mut other = member(false);
        trust_global_object(&mut other, None);
        assert_eq!(parts(&other), json!([{ "isMethod": true }]));
    }
}
