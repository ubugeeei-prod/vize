//! Inspector line-diff computation and statistics.

use vize_carton::{String, ToCompactString};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorDiff {
    pub lines: Vec<InspectorDiffLine>,
    pub stats: InspectorDiffStats,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorDiffLine {
    pub kind: &'static str,
    pub left_line: Option<usize>,
    pub right_line: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct InspectorDiffStats {
    pub additions: usize,
    pub removals: usize,
    pub unchanged: usize,
}

pub fn build_diff(left: &str, right: &str) -> InspectorDiff {
    let lines = build_line_diff(left, right);
    let stats = diff_stats(&lines);
    InspectorDiff { lines, stats }
}

pub fn build_line_diff(left: &str, right: &str) -> Vec<InspectorDiffLine> {
    let left_lines = split_diff_lines(left);
    let right_lines = split_diff_lines(right);
    let rows = left_lines.len() + 1;
    let cols = right_lines.len() + 1;
    let mut table = vec![vec![0usize; cols]; rows];

    for left_index in (0..left_lines.len()).rev() {
        for right_index in (0..right_lines.len()).rev() {
            let same_score =
                diff_line_match_weight(&left_lines[left_index], &right_lines[right_index]);
            let take_same = if same_score > 0 {
                table[left_index + 1][right_index + 1] + same_score
            } else {
                0
            };
            table[left_index][right_index] = take_same
                .max(table[left_index + 1][right_index])
                .max(table[left_index][right_index + 1]);
        }
    }

    let mut diff = Vec::new();
    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left_lines.len() && right_index < right_lines.len() {
        let same_score = diff_line_match_weight(&left_lines[left_index], &right_lines[right_index]);
        let take_same = if same_score > 0 {
            table[left_index + 1][right_index + 1] + same_score
        } else {
            0
        };
        if same_score > 0
            && take_same >= table[left_index + 1][right_index]
            && take_same >= table[left_index][right_index + 1]
        {
            diff.push(InspectorDiffLine {
                kind: "same",
                left_line: Some(left_index + 1),
                right_line: Some(right_index + 1),
                text: left_lines[left_index].clone(),
            });
            left_index += 1;
            right_index += 1;
        } else if table[left_index + 1][right_index] >= table[left_index][right_index + 1] {
            diff.push(InspectorDiffLine {
                kind: "remove",
                left_line: Some(left_index + 1),
                right_line: None,
                text: left_lines[left_index].clone(),
            });
            left_index += 1;
        } else {
            diff.push(InspectorDiffLine {
                kind: "add",
                left_line: None,
                right_line: Some(right_index + 1),
                text: right_lines[right_index].clone(),
            });
            right_index += 1;
        }
    }

    while left_index < left_lines.len() {
        diff.push(InspectorDiffLine {
            kind: "remove",
            left_line: Some(left_index + 1),
            right_line: None,
            text: left_lines[left_index].clone(),
        });
        left_index += 1;
    }

    while right_index < right_lines.len() {
        diff.push(InspectorDiffLine {
            kind: "add",
            left_line: None,
            right_line: Some(right_index + 1),
            text: right_lines[right_index].clone(),
        });
        right_index += 1;
    }

    diff
}

fn diff_line_match_weight(left: &str, right: &str) -> usize {
    if left != right {
        return 0;
    }

    let trimmed = left.trim();
    if trimmed.is_empty() {
        1
    } else if trimmed
        .chars()
        .any(|character| character.is_alphanumeric() || character == '_' || character == '$')
    {
        32 + trimmed.chars().take(80).count()
    } else {
        2 + trimmed.chars().take(8).count()
    }
}

pub fn diff_stats(lines: &[InspectorDiffLine]) -> InspectorDiffStats {
    lines
        .iter()
        .fold(InspectorDiffStats::default(), |mut stats, line| {
            match line.kind {
                "add" => stats.additions += 1,
                "remove" => stats.removals += 1,
                "same" => stats.unchanged += 1,
                _ => {}
            }
            stats
        })
}

fn split_diff_lines(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }

    value
        .replace("\r\n", "\n")
        .split('\n')
        .map(|line| line.to_compact_string())
        .collect()
}
