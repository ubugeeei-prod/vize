use super::super::NoEmptyPattern;
use super::reported;

#[test]
fn empty_pattern_reads_vue2_slot_scope_attributes() {
    assert_eq!(
        reported(
            Box::new(NoEmptyPattern),
            r#"<el-table-column><template slot-scope="{}"><i /></template><template scope="{ row }"><b /></template></el-table-column>"#,
        ),
        ["{}"]
    );
}
