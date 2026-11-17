use crate::support::fix_history_runner::check_pack;

#[test]
fn declared_slots_stay_optional_for_parents() {
    check_pack(
        "declared-slots-optional-4529",
        "fix_history_cases::declared_slots_optional_4529::declared_slots_stay_optional_for_parents",
        "4335e65ed7e650102f6cfcb0483eb20ba7773b09",
        Some(4529),
        &[
            "provided-and-omitted-declared-slots",
            "omitted-only-declared-slot",
            "unknown-declared-slot-payload-member",
        ],
    );
}
