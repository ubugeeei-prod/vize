use crate::support::fix_history_runner::check_pack;

#[test]
fn v_else_if_chain_narrows_with_linear_control_flow() {
    check_pack(
        "v-else-if-linear-control-flow-890",
        "fix_history_cases::v_else_if_linear_control_flow_890::v_else_if_chain_narrows_with_linear_control_flow",
        "f34d9e6da1aeb4ce0b641cb1a3c74c1840ca53b7",
        Some(890),
        &[
            "discriminated-union-else-if-chain",
            "status-if-else-if-else-chain",
            "else-if-branch-reads-other-variant",
        ],
    );
}
