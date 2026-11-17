//! Verified `vue.plugins` plans for in-process compiler hosts (#13074).
//!
//! A Vite host resolves `vue.plugins` once into the canonical `{ digest,
//! plan }` projection and registers it here once: the plan is re-verified
//! natively, refused when a capability is unimplemented, and its directive
//! table is lowered once. Each compile then names the plan by its digest, so
//! a request carries 64 bytes instead of the plan, and an unknown digest
//! fails closed as a stale plan.

use std::sync::{Mutex, PoisonError};

use vize_carton::config::plugin_plan::{PluginPlanError, VerifiedPluginPlan, verify_plugin_plan};
use vize_l0::config::DirectiveContracts;
use vize_l0::{String, cstr};

/// Plans one host process keeps; a config reload registers a new digest.
const MAX_PLANS: usize = 8;

struct Registered {
    plan: VerifiedPluginPlan,
    directives: DirectiveContracts,
}

static PLANS: Mutex<Vec<Registered>> = Mutex::new(Vec::new());

fn invalid(message: &str) -> PluginPlanError {
    PluginPlanError {
        code: "plan-invalid",
        subject: "plan".into(),
        message: message.into(),
    }
}

/// Verify one projected `{ digest, plan }` (JSON text) and keep it. Returns
/// the digest compile requests name it by.
pub fn register(projected: &str) -> Result<String, PluginPlanError> {
    let value: serde_json::Value =
        serde_json::from_str(projected).map_err(|error| invalid(&cstr!("{error}")))?;
    let (Some(digest), Some(plan), Some(2)) = (
        value.get("digest").and_then(serde_json::Value::as_str),
        value.get("plan"),
        value.as_object().map(serde_json::Map::len),
    ) else {
        return Err(invalid("a projected plan is { digest, plan }"));
    };
    let plan = verify_plugin_plan(plan, Some(digest))?;
    if plan.is_empty() {
        return Err(invalid("an empty plan is never projected"));
    }
    plan.refuse_unimplemented()?;
    keep(plan)
}

/// Keep a verified plan and lower its directive table once.
fn keep(plan: VerifiedPluginPlan) -> Result<String, PluginPlanError> {
    let digest = String::from(plan.digest());
    let directives = DirectiveContracts::new(plan.directives()?);
    let mut plans = PLANS.lock().unwrap_or_else(PoisonError::into_inner);
    if !plans.iter().any(|kept| kept.plan.digest() == digest) {
        if plans.len() == MAX_PLANS {
            plans.remove(0);
        }
        plans.push(Registered { plan, directives });
    }
    Ok(digest)
}

/// The directive table of the plan named `digest`, admitted for the output
/// one compile produces. No digest means no plan and an empty table.
pub fn compile_directives(
    digest: Option<&str>,
    ssr: bool,
    vapor: bool,
) -> Result<DirectiveContracts, PluginPlanError> {
    let Some(digest) = digest else {
        return Ok(DirectiveContracts::EMPTY);
    };
    let plans = PLANS.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(kept) = plans.iter().find(|kept| kept.plan.digest() == digest) else {
        return Err(PluginPlanError {
            code: "plan-stale",
            subject: "plan".into(),
            message: cstr!("digest {digest} is not registered; reload vite.config.*"),
        });
    };
    let output = if ssr {
        "ssr"
    } else if vapor {
        "vapor"
    } else {
        "vdom"
    };
    kept.plan.admit("compiler", Some(output))?;
    Ok(kept.directives.clone())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vize_carton::config::plugin_plan::verify_plugin_plan;

    use super::{compile_directives, keep, register};

    /// The declared-capabilities fixture reduced to its directive plugin.
    fn tracking(outputs: &[&str]) -> serde_json::Value {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/_fixtures/plugin-plan/resolve/declared-capabilities.json"
        ))
        .unwrap();
        let mut plugin = fixture["expect"]["plan"]["plugins"][2].clone();
        plugin["index"] = json!(0);
        plugin["supports"]["outputs"] = json!(outputs);
        json!({
            "claims": [{ "key": "vue.directive:track", "owner": "@acme/tracking" }],
            "planSchema": 1,
            "plugins": [plugin],
        })
    }

    #[test]
    fn a_compile_without_a_plan_reads_no_table() {
        assert!(compile_directives(None, false, false).unwrap().is_empty());
    }

    #[test]
    fn a_kept_plan_serves_its_table_for_supported_outputs_only() {
        let plan = verify_plugin_plan(&tracking(&["ssr", "vdom"]), None).unwrap();
        let digest = keep(plan).unwrap();
        let table = compile_directives(Some(&digest), false, false).unwrap();
        assert_eq!(
            table.get("track").map(|contract| contract.plugin.as_str()),
            Some("@acme/tracking")
        );
        assert!(compile_directives(Some(&digest), true, false).is_ok());
        assert_eq!(
            compile_directives(Some(&digest), false, true)
                .unwrap_err()
                .to_string(),
            "[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the vapor output"
        );
    }

    #[test]
    fn unknown_digests_and_malformed_projections_fail_closed() {
        assert_eq!(
            compile_directives(Some("0000"), false, false)
                .unwrap_err()
                .code,
            "plan-stale"
        );
        for projected in ["{}", r#"{"digest":"x"}"#, "not json"] {
            assert_eq!(register(projected).unwrap_err().code, "plan-invalid");
        }
        let plan = tracking(&["vdom"]);
        let stale = json!({ "digest": "0".repeat(64), "plan": plan }).to_string();
        assert_eq!(register(&stale).unwrap_err().code, "plan-stale");
    }
}
