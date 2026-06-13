use super::ReactivityIssueKind;
use vize_carton::CompactString;

#[test]
fn test_reactivity_issue_kind() {
    let kind = ReactivityIssueKind::DestructuredReactive {
        source_name: CompactString::new("state"),
        destructured_props: vec![CompactString::new("count")],
    };

    match kind {
        ReactivityIssueKind::DestructuredReactive { source_name, .. } => {
            assert_eq!(source_name.as_str(), "state");
        }
        _ => panic!("Wrong kind"),
    }
}
