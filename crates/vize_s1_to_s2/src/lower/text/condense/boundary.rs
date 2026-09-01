use vize_s1::SurfaceChild;

use super::{TextGroup, text_like};

pub(super) fn trailing_comment_padding(
    children: &[SurfaceChild<'_>],
    group_start: usize,
    lo: usize,
) -> bool {
    if group_start <= lo {
        return false;
    }
    let mut index = group_start - 1;
    if !matches!(children.get(index), Some(SurfaceChild::Comment(_))) {
        return false;
    }
    while index > lo && matches!(children.get(index - 1), Some(SurfaceChild::Comment(_))) {
        index -= 1;
    }
    if index <= lo {
        return false;
    }
    let left = index - 1;
    if !matches!(
        children.get(left),
        Some(SurfaceChild::Text(token)) if token.text.chars().all(super::is_vue_ws)
    ) {
        return false;
    }
    left > lo && text_like(&children[left - 1])
}

pub(super) fn comment_separated_element_gap_with_newline(
    children: &[SurfaceChild<'_>],
    group: &TextGroup,
    lo: usize,
    hi: usize,
) -> bool {
    if group.has_newline || group.end >= hi {
        return false;
    }
    let mut index = group.end;
    let mut saw_comment = false;
    while index < hi && matches!(children.get(index), Some(SurfaceChild::Comment(_))) {
        saw_comment = true;
        index += 1;
    }
    if !saw_comment {
        return false;
    }
    let mut has_newline = false;
    let mut saw_whitespace = false;
    while index < hi {
        match children.get(index) {
            Some(SurfaceChild::Text(token)) if token.text.chars().all(super::is_vue_ws) => {
                saw_whitespace = true;
                has_newline |= token.text.contains('\n') || token.text.contains('\r');
                index += 1;
            }
            Some(SurfaceChild::Comment(_)) => index += 1,
            _ => break,
        }
    }
    if !saw_whitespace || !has_newline {
        return false;
    }
    let prev_is_text = group.start > lo && text_like(&children[group.start - 1]);
    let next_is_text = index < hi && text_like(&children[index]);
    !prev_is_text && !next_is_text
}

pub(super) fn comments_reach_left_boundary(
    children: &[SurfaceChild<'_>],
    mut index: usize,
    lo: usize,
) -> bool {
    loop {
        if !matches!(children.get(index), Some(SurfaceChild::Comment(_))) {
            return false;
        }
        if index == lo {
            return true;
        }
        index -= 1;
    }
}

pub(super) fn comments_reach_right_boundary(
    children: &[SurfaceChild<'_>],
    mut index: usize,
    hi: usize,
) -> bool {
    while index < hi {
        if !matches!(children.get(index), Some(SurfaceChild::Comment(_))) {
            return false;
        }
        index += 1;
    }
    true
}
