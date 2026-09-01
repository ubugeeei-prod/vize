use alloc::vec::Vec as StdVec;

use vize_s2::op::{Attribute, BindOp, BindingOp, OnOp};

use super::super::EmitError;
use super::super::props::{Piece, pieces};

pub(super) enum Arg<'a> {
    Object {
        if_key: Option<&'a str>,
        pieces: StdVec<Piece<'a>>,
        suppressed_authored_key: bool,
    },
    BindSpread(&'a BindOp<'a>),
    OnSpread(&'a OnOp<'a>),
}

impl Arg<'_> {
    fn is_spread(&self) -> bool {
        matches!(self, Self::BindSpread(_) | Self::OnSpread(_))
    }
}

pub(super) fn lone_kind_spread<'a>(args: &'a [Arg<'a>]) -> Option<&'a Arg<'a>> {
    if args.is_empty() {
        return None;
    }
    let all_bind = args.iter().all(|arg| matches!(arg, Arg::BindSpread(_)));
    let all_on = args.iter().all(|arg| matches!(arg, Arg::OnSpread(_)));
    if all_bind || all_on {
        args.first()
    } else {
        None
    }
}

pub(super) fn key_and_bind_spread(args: &[Arg<'_>]) -> bool {
    matches!(
        args,
        [
            Arg::Object {
                if_key: Some(_),
                pieces,
                suppressed_authored_key: false,
                ..
            },
            Arg::BindSpread(_),
        ] if pieces.is_empty()
    )
}

pub(super) fn force_multiline_object_arg(
    args: &[Arg<'_>],
    index: usize,
    pieces: &[Piece<'_>],
    for_item: bool,
) -> bool {
    let after_spread = args[..index].iter().any(Arg::is_spread);
    if !after_spread || pieces.is_empty() {
        return pieces.len() == 1
            && for_item
            && args[index + 1..].iter().any(Arg::is_spread)
            && has_object_with_props(&args[index + 1..]);
    }
    let has_later_spread = args[index + 1..].iter().any(Arg::is_spread);
    if has_later_spread && single_static_attr_before_object_on(args, index, pieces, for_item) {
        return false;
    }
    if for_item || has_later_spread {
        return true;
    }
    if pieces.len() == 1 && has_branch_object_with_props_before_spread(&args[..index]) {
        return true;
    }
    if pieces.len() == 1 && has_unsuppressed_key_only_branch_before_spread(&args[..index]) {
        return true;
    }
    let has_branch_key = args.iter().any(|arg| {
        matches!(
            arg,
            Arg::Object {
                if_key: Some(_),
                ..
            }
        )
    });
    pieces.iter().any(|piece| match piece {
        Piece::On(_) => has_branch_key,
        Piece::Attr(attr) if matches!(attr.name, "class" | "style") => has_branch_key,
        _ => false,
    })
}

pub(super) fn merge_args<'a>(
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
    if_key: Option<&'a str>,
    skip_is: bool,
    suppress_key: bool,
) -> Result<StdVec<Arg<'a>>, EmitError> {
    let mut args = StdVec::new();
    let mut current = StdVec::new();
    let mut suppressed_authored_key = false;
    for piece in pieces(attributes, bindings, skip_is)? {
        if (if_key.is_some() || suppress_key) && piece_is_key(&piece) {
            suppressed_authored_key = true;
            continue;
        }
        match piece {
            Piece::Bind(bind) if bind.name.is_none() => {
                flush_object(&mut args, &mut current);
                args.push(Arg::BindSpread(bind));
            }
            Piece::On(on) if on.name.is_none() => {
                flush_object(&mut args, &mut current);
                args.push(Arg::OnSpread(on));
            }
            other => current.push(other),
        }
    }
    flush_object(&mut args, &mut current);
    if if_key.is_some() {
        match args.first_mut() {
            Some(Arg::Object {
                if_key: slot,
                suppressed_authored_key: suppressed,
                ..
            }) => {
                *slot = if_key;
                *suppressed = suppressed_authored_key;
            }
            _ => args.insert(
                0,
                Arg::Object {
                    if_key,
                    pieces: StdVec::new(),
                    suppressed_authored_key,
                },
            ),
        }
    }
    Ok(args)
}

fn has_branch_object_with_props_before_spread(args: &[Arg<'_>]) -> bool {
    args.iter().any(|arg| {
        matches!(
            arg,
            Arg::Object {
                if_key: Some(_),
                pieces,
                ..
            } if !pieces.is_empty()
        )
    })
}

fn has_unsuppressed_key_only_branch_before_spread(args: &[Arg<'_>]) -> bool {
    args.iter().any(|arg| {
        matches!(
            arg,
            Arg::Object {
                if_key: Some(_),
                pieces,
                suppressed_authored_key: false,
            } if pieces.is_empty()
        )
    })
}

fn has_object_with_props(args: &[Arg<'_>]) -> bool {
    args.iter().any(|arg| {
        matches!(
            arg,
            Arg::Object {
                pieces,
                ..
            } if !pieces.is_empty()
        )
    })
}

fn single_static_attr_before_object_on(
    args: &[Arg<'_>],
    index: usize,
    pieces: &[Piece<'_>],
    for_item: bool,
) -> bool {
    !for_item
        && matches!(pieces, [Piece::Attr(_)])
        && args[index + 1..]
            .iter()
            .all(|arg| matches!(arg, Arg::OnSpread(_)))
}

fn piece_is_key(piece: &Piece<'_>) -> bool {
    match piece {
        Piece::Attr(attr) => attr.name == "key",
        Piece::Bind(bind) => super::super::props_bind::is_key_bind_name(bind),
        _ => false,
    }
}

fn flush_object<'a>(args: &mut StdVec<Arg<'a>>, current: &mut StdVec<Piece<'a>>) {
    if current.is_empty() {
        return;
    }
    args.push(Arg::Object {
        if_key: None,
        pieces: core::mem::take(current),
        suppressed_authored_key: false,
    });
}
