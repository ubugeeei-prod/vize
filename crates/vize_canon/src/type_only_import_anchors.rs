//! Regression test for type-only import value anchoring in virtual TS (#1867).
//!
//! Kept as its own crate-root test module (declared in `lib.rs`) rather than in
//! `tests.rs`: that file is already over the 350-line source-length limit, so
//! adding to it trips the source-length guard. Keep new cases here instead.

use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn virtual_ts_does_not_value_anchor_type_position_imports() {
    let source = r#"<script setup lang="ts">
import { GetStudyHistoriesQuery } from './schema'
import { UnwrapArray } from './types'

type StudyHistory = UnwrapArray<
  GetStudyHistoriesQuery['student']['studyHistories']
>

defineProps<{ studyHistory: StudyHistory }>()
</script>

<template>
  <article>{{ studyHistory }}</article>
</template>"#;

    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.unwrap_or_default();

    assert!(
        !virtual_ts.contains("void GetStudyHistoriesQuery"),
        "type-only import must not be anchored as a value:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("void UnwrapArray"),
        "type helper import must not be anchored as a value:\n{virtual_ts}"
    );
}
