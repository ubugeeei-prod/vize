//! Comment-only whitespace groups keep authored decisions and boundaries.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

mod support;

use vize_s0::{Allocator, Span, String};
use vize_s1::SurfaceChild;
use vize_s1_to_s2::{Lowered, lower, lower_preserving_comments};
use vize_s2::folio::{DisegnoFolio, FolioExpr, FolioOp};

fn assert_comments(lowered: &Lowered<'_>, source: &str, comments: &[&str], preserved: bool) {
    let mut records = lowered
        .provenance
        .iter()
        .filter(|record| matches!(record.rule.as_str(), "drop.comment" | "lower.comment"));
    for comment in comments {
        let record = records.next().expect("each comment has one decision");
        let start = source.find(comment).expect("comment is authored") as u32;
        assert_eq!(record.before.as_str(), *comment);
        assert_eq!(record.span, Span::new(start, start + comment.len() as u32));
        assert_eq!(
            record.rule.as_str(),
            if preserved {
                "lower.comment"
            } else {
                "drop.comment"
            }
        );
        assert_eq!(
            record.after.as_str(),
            if preserved { "ui.comment" } else { "" }
        );
        assert_eq!(record.node.is_some(), preserved);
    }
    assert!(
        records.next().is_none(),
        "comments must not be recorded twice"
    );
}

#[test]
fn comment_only_runs_keep_one_authored_decision_per_comment() {
    let comment = "<!--注釈🦀-->";
    let count = 256usize;
    let mut source = String::from("<div>");
    for _ in 0..count {
        source.push_str(comment);
    }
    source.push_str("</div>");
    support::with_lowered(source.as_str(), |lowered, folio| {
        let [FolioOp::Element(element)] = folio.ops.as_slice() else {
            panic!("expected the div root");
        };
        assert!(element.children.is_empty());
        assert_eq!(lowered.op_count, 1);
        assert!(lowered.diagnostics.is_empty());
        assert_eq!(lowered.provenance.len(), count + 1);
        for (index, record) in lowered.provenance[1..].iter().enumerate() {
            let start = ("<div>".len() + index * comment.len()) as u32;
            assert_eq!(record.rule.as_str(), "drop.comment");
            assert_eq!(record.before.as_str(), comment);
            assert_eq!(record.after.as_str(), "");
            assert_eq!(record.node, None);
            assert_eq!(record.span, Span::new(start, start + comment.len() as u32));
        }
    });
    support::assert_transformed_sound(source.as_str(), "comment-only-run");
}

#[test]
fn comment_only_runs_stop_before_elements_and_interpolations() {
    let source = "<div><!--a--><!--b--><i/><!--c--><!--d-->{{ value }}<!--e--><!--f--></div>";
    support::with_lowered(source, |lowered, folio| {
        let [FolioOp::Element(root)] = folio.ops.as_slice() else {
            panic!("expected the div root");
        };
        let [
            FolioOp::Element(child),
            FolioOp::Interpolation(interpolation),
        ] = root.children.as_slice()
        else {
            panic!("comments must not consume the following element or interpolation");
        };
        assert_eq!(child.tag, "i");
        assert!(
            matches!(&interpolation.expression, FolioExpr::Js { source, .. } if source == "value")
        );
        assert_comments(
            lowered,
            source,
            &[
                "<!--a-->", "<!--b-->", "<!--c-->", "<!--d-->", "<!--e-->", "<!--f-->",
            ],
            false,
        );
        assert_eq!(lowered.op_count, 3);
    });
    support::assert_transformed_sound(source, "comment-run-boundaries");
}

#[test]
fn comments_before_text_remain_in_the_whitespace_group() {
    let source = "<div><!--a--><!--b-->\n  x<!--c--><!--d-->\n y</div>";
    support::with_lowered(source, |lowered, folio| {
        let [FolioOp::Element(root)] = folio.ops.as_slice() else {
            panic!("expected the div root");
        };
        assert!(
            matches!(root.children.as_slice(), [FolioOp::Text(text)] if text.content == " x y")
        );
        assert_comments(
            lowered,
            source,
            &["<!--a-->", "<!--b-->", "<!--c-->", "<!--d-->"],
            false,
        );
    });
    support::assert_transformed_sound(source, "comments-before-text");
}

#[test]
fn preserved_comments_remain_distinct_children_in_authored_order() {
    let source = "<div><!--a--><!--b--><i/><!--c--><!--d--></div>";
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    assert!(errors.is_empty());
    let lowered = lower_preserving_comments(&allocator, &tree, &errors);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    let [FolioOp::Element(root)] = folio.ops.as_slice() else {
        panic!("expected the div root");
    };
    let [
        FolioOp::Comment(a),
        FolioOp::Comment(b),
        FolioOp::Element(element),
        FolioOp::Comment(c),
        FolioOp::Comment(d),
    ] = root.children.as_slice()
    else {
        panic!("preserved comments must remain separate children");
    };
    assert_eq!(
        (
            a.content.as_str(),
            b.content.as_str(),
            element.tag.as_str(),
            c.content.as_str(),
            d.content.as_str()
        ),
        ("a", "b", "i", "c", "d")
    );
    assert_comments(
        &lowered,
        source,
        &["<!--a-->", "<!--b-->", "<!--c-->", "<!--d-->"],
        true,
    );
    assert_eq!(lowered.op_count, 6);
    support::assert_authored_artifact(source, &lowered);
}

#[test]
fn recovered_leading_bytes_still_break_comment_text_groups() {
    let source = "<div>a<!--a-->gap<!--b--> b</div>";
    let allocator = Allocator::new();
    let (mut tree, errors) = vize_s1::parse(&allocator, source);
    assert!(errors.is_empty());
    let SurfaceChild::Element(root) = &mut tree.children[0] else {
        panic!("expected the div root");
    };
    // Model an S1 recovery that keeps junk as the next token's leading
    // bytes. The authored source partition remains unchanged.
    let SurfaceChild::Text(gap) = root.children.remove(2) else {
        panic!("expected the authored gap text");
    };
    assert_eq!(gap.text, "gap");
    let SurfaceChild::Comment(comment) = &mut root.children[2] else {
        panic!("expected the following comment");
    };
    comment.leading = gap.text;
    assert_eq!(vize_s1::check_fidelity(&tree), Ok(()));
    let lowered = lower(&allocator, &tree, &errors);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    let [FolioOp::Element(root)] = folio.ops.as_slice() else {
        panic!("expected the div root");
    };
    let [FolioOp::Text(first), FolioOp::Text(last)] = root.children.as_slice() else {
        panic!("text across recovered bytes must not merge");
    };
    assert_eq!((first.content.as_str(), last.content.as_str()), ("a", " b"));
    assert_eq!(first.span, Span::new(5, 6));
    let last_start = source.find(" b").expect("last text exists") as u32;
    assert_eq!(last.span, Span::new(last_start, last_start + 2));
    assert!(lowered.texts.is_empty());
    assert_comments(&lowered, source, &["<!--a-->", "<!--b-->"], false);
    support::assert_authored_artifact(source, &lowered);
}
