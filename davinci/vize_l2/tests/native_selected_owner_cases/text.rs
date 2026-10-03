use super::*;

#[test]
fn original_root_html_whitespace_is_unavailable_before_text_mint() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template> </template>",
        "<template>\t</template>",
        "<template>\n</template>",
        "<template>\r</template>",
        "<template>\x0c</template>",
        "<template>a  b</template>",
        "<template> a </template>",
    ] {
        let mut owner = owner(&arena, source)?;
        {
            let mut walk = owner.begin().map_err(|_| "original start")?;
            let child = walk.selected().children().next().ok_or("original text")?;
            check(matches!(walk.child(child), Err(issue) if issue.kind == Kind::UnsupportedChild))?;
            check(walk.complete().is_err())?;
        }
        let original = owner.finish();
        check(original.view().is_err())?;
        let file = original.file().ok_or("actual incomplete File")?;
        check(file.artifact().node_count() == 0 && !file.is_complete())?;
        check(core::ptr::eq(file.artifact().source(), source))?;
    }
    Ok(())
}

#[test]
fn root_text_boundary_retains_completed_siblings_unicode_and_original_comment()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<template>original<!--same comment--> a </template>";
    let mut prefix_owner = owner(&arena, source)?;
    {
        let mut walk = prefix_owner.begin().map_err(|_| "start")?;
        let mut children = walk.selected().children();
        walk.child(children.next().ok_or("first")?)
            .map_err(|_| "original text")?;
        walk.child(children.next().ok_or("second")?)
            .map_err(|_| "original comment")?;
        check(walk.child(children.next().ok_or("third")?).is_err())?;
        check(walk.complete().is_err())?;
    }
    let result = prefix_owner.finish();
    check(result.view().is_err())?;
    let file = result.file().ok_or("partial File")?;
    let [
        vize_l2::op::Op::Text(first),
        vize_l2::op::Op::Comment(comment),
    ] = file.artifact().root().ops.as_slice()
    else {
        return Err("completed actual siblings");
    };
    check(first.content == "original" && comment.content == "same comment")?;
    for source in [
        "<template>hé\u{a0}tail</template>",
        "<template><!-- a b --></template>",
        "<template></template>",
    ] {
        let mut owner = owner(&arena, source)?;
        {
            let mut walk = owner.begin().map_err(|_| "native start")?;
            for child in walk.selected().children() {
                walk.child(child).map_err(|_| "eligible child")?;
            }
            walk.complete().map_err(|_| "real complete")?;
        }
        check(owner.finish().view().is_ok())?;
    }
    Ok(())
}
