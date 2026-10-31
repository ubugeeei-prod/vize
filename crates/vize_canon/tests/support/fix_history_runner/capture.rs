//! Only the dedicated zero-argument admission may omit canonical observations.

use super::super::fix_history_observation::Capture;

#[derive(Clone, Copy)]
pub(crate) struct WorkspaceCapturePolicy {
    pub(crate) record: bool,
}

impl WorkspaceCapturePolicy {
    pub(super) fn capture(self, pack: &str, test: &str) -> Option<Capture> {
        if self.record {
            return Capture::from_environment();
        }
        assert_eq!(pack, "zero-argument-slot-constructor-11013");
        assert_eq!(
            test,
            "zero_argument_slot_constructors_preserve_mapper_and_authored_diagnostics"
        );
        let executable =
            std::env::current_exe().expect("dedicated fixture executable must resolve");
        let name = executable
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("dedicated fixture executable name must be UTF-8");
        let hash = name
            .strip_prefix("zero_argument_slot_constructor_diagnostics-")
            .expect("only the actual dedicated fixture executable may omit workspace recording");
        assert!(!hash.is_empty() && hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
        None
    }
}
