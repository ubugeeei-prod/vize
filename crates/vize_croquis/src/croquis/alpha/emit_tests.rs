//! Every authored event overload participates in its declaration contract.

use super::EmitContract;
use crate::{Croquis, Drawer, DrawerOptions};
use vize_l2::summary::{Facet, SfcSummary};

fn draw(source: &str) -> Croquis {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(source);
    drawer.finish()
}

fn summary(croquis: &Croquis) -> SfcSummary {
    SfcSummary::from_alpha(croquis.alpha_pages("Component", None).expect("pages")).expect("summary")
}

#[test]
fn later_event_overload_edits_dirty_only_their_event_and_signature() {
    let source = "type Later = string; defineProps<{ label: string }>(); \
        defineSlots<{ header(props: {}): void }>(); \
        defineEmits<{ (event: 'save', value: number): void; \
        (event: 'save', value: Later): void; stable: [] }>();";
    let before = draw(source);
    let after = draw(&source.replace("Later = string", "Later = boolean"));
    let changed = summary(&before).changed(&summary(&after));
    assert!(
        changed
            .iter()
            .any(|id| id.facet() == Facet::Emit && id.name() == "save")
    );
    assert!(
        !changed
            .iter()
            .any(|id| matches!(id.facet(), Facet::Prop | Facet::Slot)
                || id.facet() == Facet::Emit && id.name() == "stable")
    );
    let pages = before.alpha_pages("Component", None).expect("pages");
    let save: EmitContract = serde_json::from_str(
        &pages
            .emits
            .iter()
            .find(|entry| entry.name == "save")
            .expect("save")
            .contract,
    )
    .expect("contract");
    assert_eq!(save.payload.as_deref(), Some("[value: number]"));
    assert_eq!(
        save.overload_payloads
            .iter()
            .map(|payload| payload.as_deref())
            .collect::<Vec<_>>(),
        [Some("[value: number]"), Some("[value: Later]")]
    );
    assert!(save.type_dependencies.complete);
    assert!(
        save.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Later")
    );
}

#[test]
fn unknown_overloads_do_not_erase_known_payload_dependencies() {
    let croquis = draw(
        "type Public = string; defineEmits<{ \
        (event: 'save', value: Public): void; \
        <T>(event: 'save', value: T): void }>();",
    );
    let pages = croquis.alpha_pages("Component", None).expect("pages");
    let save: EmitContract = serde_json::from_str(&pages.emits[0].contract).expect("contract");
    assert_eq!(save.overload_payloads.len(), 2);
    assert_eq!(save.overload_payloads[1], None);
    assert!(!save.type_dependencies.complete);
    assert!(
        save.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Public")
    );
}

#[test]
fn unresolved_generic_event_annotations_remain_observable() {
    let source = "type Bound = { id: string }; defineProps<{ label: string }>(); \
        defineSlots<{ header(props: {}): void }>(); \
        defineEmits<{ <T extends Bound>(event: 'save', value: T): void; stable: [] }>();";
    let before = draw(source);
    for changed in [
        source.replace("value: T", "value: string"),
        source.replace("id: string", "id: number"),
    ] {
        let after = draw(&changed);
        assert_ne!(
            summary(&before).fingerprint(Facet::Emit, "save"),
            summary(&after).fingerprint(Facet::Emit, "save")
        );
        for (facet, name) in [
            (Facet::Prop, "label"),
            (Facet::Slot, "header"),
            (Facet::Emit, "stable"),
        ] {
            assert_eq!(
                summary(&before).fingerprint(facet, name),
                summary(&after).fingerprint(facet, name)
            );
        }
    }
    let pages = before.alpha_pages("Component", None).expect("pages");
    let save: EmitContract = serde_json::from_str(
        &pages
            .emits
            .iter()
            .find(|entry| entry.name == "save")
            .expect("save")
            .contract,
    )
    .expect("contract");
    assert_eq!(save.payload, None);
    assert_eq!(save.overload_payloads, [None]);
    assert!(
        save.unresolved_type_arguments
            .as_deref()
            .is_some_and(|source| source.contains("<T extends Bound>"))
    );
    assert!(!save.type_dependencies.complete);
}

#[test]
fn generic_runtime_validator_contracts_follow_types_and_exclude_implementation() {
    let source = "type Bound = { id: string }; \
        const shared = { save: <T extends Bound>(value: T, count: number = 1): boolean => { return true } }; \
        const privateConfig = { save: (ignored: string): boolean => false }; \
        defineProps<{ label: string }>(); defineSlots<{ header(props: {}): void }>(); \
        defineEmits({ ...shared, stable: (value: number): boolean => true });";
    let before = draw(source);
    for edit in [
        source.replace("id: string", "id: number"),
        source.replace("value: T", "value: string"),
    ] {
        let after = draw(&edit);
        assert_ne!(
            summary(&before).fingerprint(Facet::Emit, "save"),
            summary(&after).fingerprint(Facet::Emit, "save")
        );
        for (facet, name) in [
            (Facet::Prop, "label"),
            (Facet::Slot, "header"),
            (Facet::Emit, "stable"),
        ] {
            assert_eq!(
                summary(&before).fingerprint(facet, name),
                summary(&after).fingerprint(facet, name)
            );
        }
    }
    let body_edit = source
        .replace("return true", "return value.id.length > count")
        .replace("number = 1", "number = 42")
        .replace("ignored: string", "ignored: boolean");
    assert!(
        summary(&before)
            .changed(&summary(&draw(&body_edit)))
            .is_empty()
    );
    let pages = before.alpha_pages("Component", None).expect("pages");
    let save: EmitContract = serde_json::from_str(
        &pages
            .emits
            .iter()
            .find(|entry| entry.name == "save")
            .expect("save")
            .contract,
    )
    .expect("contract");
    assert_eq!(save.payload, None);
    assert_eq!(save.unresolved_type_arguments, None);
    assert_eq!(
        save.validator_signatures,
        ["<T extends Bound>(value: T, count?: number): boolean"]
    );
    assert!(!save.type_dependencies.complete);
    assert!(
        save.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Bound")
    );
    assert!(
        !save
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "T")
    );
}

#[test]
fn runtime_validator_assertion_annotations_and_dependencies_remain_observable() {
    let source = "type Public = string; \
        const shared = { save: ((value) => true) as (value: Public) => boolean }; \
        const privateConfig = { save: ((value) => true) as (value: number) => boolean }; \
        defineProps<{ label: string }>(); \
        defineEmits({ ...shared, stable: (value: number): boolean => true });";
    let before = draw(source);
    for edit in [
        source.replace("value: Public", "value: number"),
        source.replace("Public = string", "Public = boolean"),
    ] {
        let after = draw(&edit);
        assert_ne!(
            summary(&before).fingerprint(Facet::Emit, "save"),
            summary(&after).fingerprint(Facet::Emit, "save")
        );
        for (facet, name) in [(Facet::Prop, "label"), (Facet::Emit, "stable")] {
            assert_eq!(
                summary(&before).fingerprint(facet, name),
                summary(&after).fingerprint(facet, name)
            );
        }
    }
    let body_edit = source
        .replace("(value) => true", "(value) => Boolean(value)")
        .replace("as (value: number)", "as (value: Date)");
    assert!(
        summary(&before)
            .changed(&summary(&draw(&body_edit)))
            .is_empty()
    );
    let pages = before.alpha_pages("Component", None).expect("pages");
    let save: EmitContract = serde_json::from_str(
        &pages
            .emits
            .iter()
            .find(|entry| entry.name == "save")
            .expect("save")
            .contract,
    )
    .expect("contract");
    assert_eq!(save.payload, None);
    assert_eq!(
        save.validator_type_annotations,
        ["(value: Public) => boolean"]
    );
    assert!(!save.type_dependencies.complete);
    assert!(
        save.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Public")
    );
    assert!(
        !save
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Date")
    );
}

#[test]
fn const_asserted_runtime_literals_and_spreads_keep_real_payload_dependencies() {
    let prefix = "type Public = { id: string }; type Contract = { save: (value: Public) => boolean; ready: () => boolean };";
    let literal = "{ save: (value: Public): boolean => true, ready: (): boolean => true }";
    for source in [
        format!("{prefix} defineEmits(({literal}) as const);"),
        format!("{prefix} const shared = ({literal}) as const; defineEmits({{ ...shared }});"),
    ] {
        let before = draw(&source);
        let pages = before.alpha_pages("Component", None).expect("pages");
        let contracts: Vec<EmitContract> = pages
            .emits
            .iter()
            .map(|entry| serde_json::from_str(&entry.contract).expect("contract"))
            .collect();
        assert_eq!(contracts.len(), 2);
        for contract in &contracts {
            assert!(contract.type_dependencies.complete);
            assert!(
                contract
                    .validator_type_annotations
                    .iter()
                    .any(|annotation| annotation == "const")
            );
            assert!(
                !contract
                    .type_dependencies
                    .declarations
                    .iter()
                    .any(|dependency| dependency.name == "const")
            );
            if contract.name == "save" {
                assert_eq!(contract.payload.as_deref(), Some("[value: Public]"));
                assert!(
                    contract
                        .type_dependencies
                        .declarations
                        .iter()
                        .any(|dependency| dependency.name == "Public")
                );
            } else {
                assert_eq!(contract.name, "ready");
                assert_eq!(contract.payload.as_deref(), Some("[]"));
            }
        }
        let changed = draw(&source.replace("id: string", "id: number"));
        assert_ne!(
            summary(&before).fingerprint(Facet::Emit, "save"),
            summary(&changed).fingerprint(Facet::Emit, "save")
        );
        assert_eq!(
            summary(&before).fingerprint(Facet::Emit, "ready"),
            summary(&changed).fingerprint(Facet::Emit, "ready")
        );
        let cast = draw(&source.replace("as const", "as Contract"));
        for entry in cast.alpha_pages("Component", None).expect("pages").emits {
            let contract: EmitContract = serde_json::from_str(&entry.contract).expect("contract");
            assert_eq!(contract.payload, None);
            assert!(!contract.type_dependencies.complete);
            assert!(
                contract
                    .type_dependencies
                    .declarations
                    .iter()
                    .any(|dependency| dependency.name == "Contract")
            );
        }
    }
}
