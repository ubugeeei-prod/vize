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
    plan.admit("compiler", None)?;
    let digest = String::from(plan.digest());
    let directives = plan.directive_table()?;
    // Every distinct plan stays for the process: a host keeps naming its
    // digest, and only a config change registers another plan.
    let mut plans = PLANS.lock().unwrap_or_else(PoisonError::into_inner);
    if !plans.iter().any(|kept| kept.plan.digest() == digest) {
        plans.push(Registered { plan, directives });
    }
    Ok(digest)
}

/// The directive table of the plan named `digest`. The table carries each
/// declaring plugin's supported outputs, so the SFC compiler refuses a file
/// whose resolved output a plugin lacks. No digest means an empty table.
pub fn compile_directives(digest: Option<&str>) -> Result<DirectiveContracts, PluginPlanError> {
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
        assert!(compile_directives(None).unwrap().is_empty());
    }

    #[test]
    fn a_kept_plan_serves_its_table_with_the_supported_outputs() {
        let plan = verify_plugin_plan(&tracking(&["ssr", "vdom"]), None).unwrap();
        let digest = keep(plan).unwrap();
        let table = compile_directives(Some(&digest)).unwrap();
        assert_eq!(
            table.get("track").map(|contract| contract.plugin.as_str()),
            Some("@acme/tracking")
        );
        assert_eq!(table.unsupported("ssr"), None);
        assert_eq!(
            table.unsupported("vapor").as_deref(),
            Some(
                "[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the vapor output"
            )
        );
    }

    #[test]
    fn unknown_digests_and_malformed_projections_fail_closed() {
        assert_eq!(
            compile_directives(Some("0000")).unwrap_err().code,
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
