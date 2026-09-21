//! TS-32's building blocks (P3-13), pinned exactly: the `[remarks-corpus]`
//! baseline page (canonical text, round trip, rejections) and the keyed
//! remarks-diff's classification of every change class.

use vize_davinci::folio::remarks::corpus::{CorpusRemark, ExplainedRegression, RemarkCorpus};
use vize_davinci::folio::remarks::diff::{ChangeKind, diff_corpus};
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_davinci::pass::RemarkKind;
use vize_davinci::pass::observer::{RecordedArg, RecordedRemark, RemarkArgValue};
use vize_s0::{Span, String};

fn remark(kind: RemarkKind, name: &str, span: (u32, u32), blocker: Option<&str>) -> RecordedRemark {
    RecordedRemark {
        stage: String::from("s2"),
        pass: String::from("hoist-static"),
        kind,
        name: String::from(name),
        span: Span::new(span.0, span.1),
        args: blocker
            .map(|blocker| RecordedArg {
                key: String::from("blocker"),
                value: RemarkArgValue::Str(String::from(blocker)),
            })
            .into_iter()
            .collect(),
    }
}

fn at(path: &str, remark: RecordedRemark) -> CorpusRemark {
    CorpusRemark {
        path: String::from(path),
        remark,
    }
}

const PAGE: &str = "[remarks-corpus]\n\n\
[remarks-corpus.files]\n\"a.vue\"\n\"dir with space/b.vue\"\n\n\
[remarks-corpus.entries]\n\
\"a.vue\" s2.hoist-static applied static-subtree @1:9\n\
\"dir with space/b.vue\" s2.hoist-static missed static-props @3:4 blocker=\"child\"\n\n\
[remarks-corpus.explained]\n\
\"dir with space/b.vue\" s2.hoist-static missed static-props @3:4 reason=\"accepted: \\\"why\\\"\"\n\n";

fn corpus() -> RemarkCorpus {
    RemarkCorpus {
        files: vec![String::from("a.vue"), String::from("dir with space/b.vue")],
        entries: vec![
            at(
                "a.vue",
                remark(RemarkKind::Applied, "static-subtree", (1, 9), None),
            ),
            at(
                "dir with space/b.vue",
                remark(RemarkKind::Missed, "static-props", (3, 4), Some("child")),
            ),
        ],
        explained: vec![ExplainedRegression {
            path: String::from("dir with space/b.vue"),
            stage: String::from("s2"),
            pass: String::from("hoist-static"),
            name: String::from("static-props"),
            span: Span::new(3, 4),
            reason: String::from("accepted: \"why\""),
        }],
    }
}

#[test]
fn the_baseline_page_prints_canonically_and_round_trips() {
    let value = corpus();
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), PAGE);
    assert_eq!(RemarkCorpus::parse(PAGE), Ok(value.clone()));
    assert_eq!(
        RemarkCorpus::default()
            .print_to_string(FolioMode::Full)
            .as_str(),
        "[remarks-corpus]\n\n"
    );
    let explained = &value.explained[0];
    assert!(explained.names("dir with space/b.vue", &value.entries[1].remark));
    assert!(!explained.names("a.vue", &value.entries[1].remark));
}

#[test]
fn malformed_baseline_lines_are_rejected_exactly() {
    let cases: [(&str, FolioError); 4] = [
        (
            "[remarks-corpus]\n\n[remarks-corpus.files]\na.vue\n",
            FolioError::new(
                4,
                String::from("corpus line `a.vue` does not start with a quoted path"),
            ),
        ),
        (
            "[remarks-corpus]\n\n[remarks-corpus.files]\n\"a.vue\" extra\n",
            FolioError::new(4, String::from("unexpected `extra` after a file path")),
        ),
        (
            "[remarks-corpus]\n\n[remarks-corpus.explained]\n\"a.vue\" s2.p applied n @1:2 reason=\"r\"\n",
            FolioError::new(
                4,
                String::from(
                    "an explained regression is a missed entry with exactly `reason=\"...\"`",
                ),
            ),
        ),
        (
            "[remarks-corpus]\n\n[remarks-corpus.lines]\n",
            FolioError::new(3, String::from("unknown section [remarks-corpus.lines]")),
        ),
    ];
    for (input, error) in cases {
        assert_eq!(RemarkCorpus::parse(input), Err(error), "{input}");
    }
}

#[test]
fn the_diff_classifies_every_change_class() {
    use RemarkKind::{Analysis, Applied, Missed};
    let before = vec![
        at("a.vue", remark(Applied, "static-subtree", (0, 10), None)),
        at(
            "a.vue",
            remark(Missed, "static-props", (0, 10), Some("child")),
        ),
        at("a.vue", remark(Analysis, "fact", (2, 3), None)),
        at(
            "a.vue",
            remark(Missed, "static-props", (4, 5), Some("binding")),
        ),
        at("b.vue", remark(Applied, "static-subtree", (1, 2), None)),
        at("b.vue", remark(Missed, "static-subtree", (1, 2), None)),
    ];
    let after = vec![
        at(
            "a.vue",
            remark(Missed, "static-subtree", (0, 10), Some("child")),
        ),
        at("a.vue", remark(Applied, "static-props", (0, 10), None)),
        at("a.vue", remark(Missed, "fact", (2, 3), None)),
        at(
            "a.vue",
            remark(Missed, "static-props", (4, 5), Some("ref-attribute")),
        ),
        // b.vue's first occurrence is unchanged; its second vanished.
        at("b.vue", remark(Applied, "static-subtree", (1, 2), None)),
        at("c.vue", remark(Applied, "static-subtree", (0, 1), None)),
    ];
    // Identity order: path, then stage/pass/name, then span, then the
    // occurrence index; changed and removed first, added after.
    let classes: Vec<(ChangeKind, String, String, (u32, u32))> = diff_corpus(&before, &after)
        .into_iter()
        .map(|change| {
            let side = change.after.or(change.before).expect("one side exists");
            (
                change.kind,
                change.path,
                side.name,
                (side.span.start, side.span.end),
            )
        })
        .collect();
    let expected =
        |kind, path: &str, name: &str, span| (kind, String::from(path), String::from(name), span);
    assert_eq!(
        classes,
        vec![
            expected(ChangeKind::Rekinded, "a.vue", "fact", (2, 3)),
            expected(ChangeKind::Improved, "a.vue", "static-props", (0, 10)),
            expected(ChangeKind::Reargued, "a.vue", "static-props", (4, 5)),
            expected(ChangeKind::Regressed, "a.vue", "static-subtree", (0, 10)),
            expected(ChangeKind::Removed, "b.vue", "static-subtree", (1, 2)),
            expected(ChangeKind::Added, "c.vue", "static-subtree", (0, 1)),
        ]
    );
    assert_eq!(diff_corpus(&before, &before), vec![]);
    assert_eq!(
        [
            ChangeKind::Regressed,
            ChangeKind::Improved,
            ChangeKind::Rekinded,
            ChangeKind::Reargued,
            ChangeKind::Added,
            ChangeKind::Removed,
        ]
        .map(ChangeKind::as_str),
        [
            "regressed",
            "improved",
            "rekinded",
            "reargued",
            "added",
            "removed"
        ]
    );
}
