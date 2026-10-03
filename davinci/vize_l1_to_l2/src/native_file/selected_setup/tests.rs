extern crate std;

use super::*;
use crate::native_file::{NativeSelectedSfcIssueKind as Kind, lower_selected_sfc_native};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{SurfaceParseOptions, container::vue::ScriptRole};
mod accepted;
mod refusal;
fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
#[track_caller]
fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required original setup envelope condition")
    }
}
