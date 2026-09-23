//! P5-4b: a source-body edit may recheck the interface, but it must not
//! execute a dependent file's query when no used declaration changed.

use vize_davinci::summary::{AlphaEntry, AlphaPages, Facet, Fingerprint, Signature};
use vize_resident::{
    Accounting, DeclarationName, QueryCounts, ResidentDatabase, ResidentSummaryError, SourceFile,
    SummaryCachePolicy, SummaryInput, declaration_fingerprint, sfc_summary,
};
use vize_s0::String;

#[salsa::tracked(returns(copy))]
fn dependent_use<'db>(
    db: &'db dyn salsa::Database,
    dependent: SourceFile,
    provider: SummaryInput,
    facet: Facet,
    name: DeclarationName<'db>,
) -> Option<Fingerprint> {
    let _path = dependent.path(db);
    declaration_fingerprint(db, provider, facet, name)
}

fn pages(prop_type: &str, emit_type: &str) -> AlphaPages {
    AlphaPages {
        signature: Signature {
            name: String::from("Button"),
            params: String::from(""),
        },
        props: vec![AlphaEntry {
            name: String::from("label"),
            contract: String::from(prop_type),
        }],
        emits: vec![AlphaEntry {
            name: String::from("select"),
            contract: String::from(emit_type),
        }],
        slots: vec![],
        reactivity: vec![],
        components: vec![],
    }
}

fn counts(rows: &[(&str, u32, u32)]) -> Accounting {
    rows.iter()
        .map(|&(name, executed, reused)| (String::from(name), QueryCounts { executed, reused }))
        .collect()
}

fn read_users(
    db: &ResidentDatabase,
    provider: SummaryInput,
    prop_user: SourceFile,
    emit_user: SourceFile,
) -> (Fingerprint, Fingerprint) {
    let label = db.declaration_name("label");
    let select = db.declaration_name("select");
    (
        dependent_use(db, prop_user, provider, Facet::Prop, label).expect("label"),
        dependent_use(db, emit_user, provider, Facet::Emit, select).expect("select"),
    )
}

#[test]
fn a_body_edit_rechecks_the_summary_without_executing_dependents() {
    let mut db = ResidentDatabase::default();
    let file = db.open("Button.vue", "<script setup>const n = 1</script>");
    let provider = db.publish_alpha(file, pages("string", "MouseEvent"));
    let prop_user = db.open("PropUser.vue", "<Button label='a' />");
    let emit_user = db.open("EmitUser.vue", "<Button @select='onSelect' />");
    let first = read_users(&db, provider, prop_user, emit_user);
    assert_eq!(
        db.take_accounting(),
        counts(&[
            ("sfc_summary", 1, 0),
            ("declaration_fingerprint", 2, 0),
            ("dependent_use", 2, 0),
        ])
    );

    db.edit_with_alpha(
        file,
        provider,
        "<script setup>const n = 2</script>",
        pages("string", "MouseEvent"),
    );
    assert_eq!(read_users(&db, provider, prop_user, emit_user), first);
    assert_eq!(
        db.take_accounting(),
        counts(&[
            ("sfc_summary", 1, 0),
            ("declaration_fingerprint", 0, 2),
            ("dependent_use", 0, 2),
        ])
    );

    db.revise_alpha(provider, pages("number", "MouseEvent"));
    let changed = read_users(&db, provider, prop_user, emit_user);
    assert_ne!(changed.0, first.0);
    assert_eq!(changed.1, first.1);
    assert_eq!(
        db.take_accounting(),
        counts(&[
            ("sfc_summary", 1, 0),
            ("declaration_fingerprint", 2, 0),
            ("dependent_use", 1, 1),
        ])
    );
}

#[test]
fn tsconfig_and_dependency_durability_do_not_recheck_on_buffer_edits() {
    let mut db = ResidentDatabase::default();
    let dependency = db.open_dependency("node_modules/ui/Button.vue", "<template>ok</template>");
    let source = db.open("App.vue", "<template>one</template>");
    let provider = db.publish_alpha(dependency, pages("string", "MouseEvent"));
    let name = db.declaration_name("label");
    let before = declaration_fingerprint(&db, provider, Facet::Prop, name);
    assert_eq!(
        db.take_accounting(),
        counts(&[("sfc_summary", 1, 0), ("declaration_fingerprint", 1, 0)])
    );

    db.edit(source, "<template>two</template>");
    let name = db.declaration_name("label");
    assert_eq!(
        declaration_fingerprint(&db, provider, Facet::Prop, name),
        before
    );
    assert_eq!(
        db.take_accounting(),
        counts(&[("sfc_summary", 0, 1), ("declaration_fingerprint", 0, 1)])
    );

    db.configure_tsconfig("{\"strict\":true}");
    let name = db.declaration_name("label");
    assert_eq!(
        declaration_fingerprint(&db, provider, Facet::Prop, name),
        None
    );
    assert_eq!(
        sfc_summary(&db, provider),
        &Err(ResidentSummaryError::StaleAlpha)
    );
    db.revise_alpha(provider, pages("string", "MouseEvent"));
    let name = db.declaration_name("label");
    assert_eq!(
        declaration_fingerprint(&db, provider, Facet::Prop, name),
        before
    );
    assert_eq!(
        sfc_summary(&db, provider)
            .as_ref()
            .expect("valid alpha")
            .len(),
        3
    );
}

#[test]
fn an_unpublished_alpha_refresh_never_serves_a_stale_fingerprint() {
    let mut db = ResidentDatabase::default();
    let file = db.open("Button.vue", "<script setup>const n = 1</script>");
    let provider = db.publish_alpha(file, pages("string", "MouseEvent"));
    let user = db.open("User.vue", "<Button label='a' />");
    let name = db.declaration_name("label");
    let before =
        dependent_use(&db, user, provider, Facet::Prop, name).expect("initial fingerprint");
    let _ = db.take_accounting();

    db.edit(file, "<script setup>const n: number = 2</script>");
    let name = db.declaration_name("label");
    assert_eq!(dependent_use(&db, user, provider, Facet::Prop, name), None);
    assert_eq!(
        sfc_summary(&db, provider),
        &Err(ResidentSummaryError::StaleAlpha)
    );
    assert_eq!(
        db.take_accounting(),
        counts(&[
            ("sfc_summary", 1, 0),
            ("declaration_fingerprint", 1, 0),
            ("dependent_use", 1, 0),
        ])
    );

    db.revise_alpha(provider, pages("number", "MouseEvent"));
    let name = db.declaration_name("label");
    let after =
        dependent_use(&db, user, provider, Facet::Prop, name).expect("refreshed fingerprint");
    assert_ne!(after, before);
}

#[test]
fn cache_policy_is_read_from_the_recorded_resource_preset() {
    assert_eq!(
        SummaryCachePolicy::from_resource_preset("linux-x64-ci"),
        Ok(SummaryCachePolicy {
            summaries: 128,
            declarations: 512,
        })
    );
    assert_eq!(
        SummaryCachePolicy::from_resource_preset("unknown").unwrap_err(),
        "missing resource summary_cache policy"
    );
}
