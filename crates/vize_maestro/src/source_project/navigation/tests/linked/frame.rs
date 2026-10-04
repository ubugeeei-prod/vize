use super::{NavigationRefusal, Position, SOURCE, block_on, project, ranges, uri, worker};

mod lifecycle;

#[test]
fn original_3471_frame_open_close_and_end_requeries_keep_the_same_original_owners() {
    let (_, project) = project(SOURCE);
    for position in [
        Position::new(4, 1),
        Position::new(4, 3),
        Position::new(4, 9),
        Position::new(9, 2),
        Position::new(9, 10),
    ] {
        assert_eq!(
            block_on(project.linked_editing(&uri(), position)),
            Ok(ranges((4, 1, 9), (9, 2, 10)))
        );
    }
    let owner = worker(&project);
    let before = block_on(owner.names_inspect()).unwrap();
    for position in [
        Position::new(6, 6),
        Position::new(9, 3),
        Position::new(5, 4),
        Position::new(4, 3),
    ] {
        block_on(project.linked_editing(&uri(), position)).unwrap();
        assert_eq!(before, block_on(owner.names_inspect()).unwrap());
    }
    assert_eq!(before.productions, 1);
    assert!(project.workers.lock().is_empty());
    assert!(project.selected_workers.lock().is_empty());
}

#[test]
fn original_nonzero_unicode_crlf_frame_and_empty_body_keep_complete_utf16_ranges() {
    let source = "<!--😀日本語-->\r\n<style>.x{}</style>\r\n<script setup lang=ts>const x=1</script>\r\n<template>\r\n<Card-é>😀</Card-é>\r\n</template \t>";
    let (_, project) = project(source);
    for position in [
        Position::new(3, 1),
        Position::new(3, 9),
        Position::new(5, 2),
        Position::new(5, 10),
    ] {
        assert_eq!(
            block_on(project.linked_editing(&uri(), position)),
            Ok(ranges((3, 1, 9), (5, 2, 10)))
        );
    }
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(4, 2))),
        Ok(ranges((4, 1, 7), (4, 12, 18)))
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(4, 9))),
        Err(NavigationRefusal::Position)
    );
    for position in [
        Position::new(0, 1),
        Position::new(1, 2),
        Position::new(2, 2),
        Position::new(3, 0),
        Position::new(5, 11),
    ] {
        assert_eq!(block_on(project.linked_editing(&uri(), position)), Ok(None));
    }
    let (_, empty) = self::project("<template></template>");
    assert_eq!(
        block_on(empty.linked_editing(&uri(), Position::new(0, 1))),
        Ok(ranges((0, 1, 9), (0, 12, 20)))
    );
}

#[test]
fn original_nested_template_and_comment_lookalikes_do_not_supply_the_outer_pair() {
    let source = "<template><!-- </template> --><template>nested</template></template>";
    let (_, project) = project(source);
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 1))),
        Ok(ranges((0, 1, 9), (0, 59, 67)))
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 31))),
        Ok(ranges((0, 31, 39), (0, 48, 56)))
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 15))),
        Ok(None)
    );
    assert_eq!(worker(&project).sfc_productions(), 1);
}

#[test]
fn original_component_descriptor_and_frame_case_refusals_remain_whole_and_sticky() {
    for source in [
        "<template><p></p><div title='unterminated</template>",
        "<template/>",
        "<template>x",
        "<template src='other.html'></template>",
    ] {
        let (_, project) = project(source);
        assert_eq!(
            block_on(project.linked_editing(&uri(), Position::new(0, 1))),
            Err(NavigationRefusal::TemplateNamesProducer)
        );
        assert_eq!(
            block_on(project.linked_editing(&uri(), Position::new(0, 2))),
            Err(NavigationRefusal::TemplateNamesProducer)
        );
        assert_eq!(worker(&project).sfc_productions(), 1);
    }
    let (_, mismatch) = project("<template><p></p></TeMPLATE >");
    for position in [Position::new(0, 1), Position::new(0, 19)] {
        assert_eq!(
            block_on(mismatch.linked_editing(&uri(), position)),
            Ok(None)
        );
    }
    assert_eq!(
        block_on(mismatch.linked_editing(&uri(), Position::new(0, 11))),
        Ok(ranges((0, 11, 12), (0, 15, 16)))
    );
}

#[test]
fn original_deep_body_is_completed_before_publishing_the_frame_pair() {
    let source = vize_l0::cstr!(
        "<template>{}x{}</template>",
        "<p>".repeat(2048),
        "</p>".repeat(2048)
    );
    let (_, project) = project(&source);
    let close = 10 + 3 * 2048 + 1 + 4 * 2048 + 2;
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 1))),
        Ok(ranges((0, 1, 9), (0, close, close + 8)))
    );
    assert_eq!(worker(&project).sfc_productions(), 1);
}
