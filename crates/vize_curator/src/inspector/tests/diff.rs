use super::super::{build_diff, build_line_diff};

#[test]
fn builds_line_diff_and_stats() {
    let diff = build_diff("one\ntwo\nthree", "one\nTWO\nthree\nfour");

    assert_eq!(diff.stats.additions, 2);
    assert_eq!(diff.stats.removals, 1);
    assert_eq!(diff.stats.unchanged, 2);
    assert_eq!(diff.lines.len(), 5);
    assert_eq!(diff.lines[0].kind, "same");
    assert_eq!(diff.lines[1].kind, "remove");
    assert_eq!(diff.lines[2].kind, "add");
    assert_eq!(diff.lines[4].right_line, Some(4));
}

#[test]
fn line_diff_prefers_content_matches_over_empty_line_anchors() {
    let left = "\
import { defineComponent as _defineComponent } from 'vue'
import { computed, watch } from 'vue'

// Reactive Props Destructure
export default {}";
    let right = "\
import { defineComponent as _defineComponent } from 'vue'
import {
  openBlock as _openBlock,
} from 'vue'

import { computed, watch } from 'vue'

export default {}";

    let diff = build_line_diff(left, right);
    let matched_import = diff
        .iter()
        .find(|line| line.text == "import { computed, watch } from 'vue'")
        .expect("matching import line exists");

    assert_eq!(matched_import.kind, "same");
    assert_eq!(matched_import.left_line, Some(2));
    assert_eq!(matched_import.right_line, Some(6));
    assert!(!diff.iter().any(|line| {
        line.kind == "remove" && line.text == "import { computed, watch } from 'vue'"
    }));
}
