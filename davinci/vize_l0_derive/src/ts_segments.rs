//! Reserve each `ts!` segment exactly once.
//!
//! A run of static text, holes, and repeats with no `#{ }` is one reserve.
//! `#{ }` breaks the run so the statement can write through `__ts` and the
//! following text reserves only its own bytes. Repeats that contain `#{ }`
//! reserve inside each iteration, after the statement has run.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::emit_ts::Piece;

pub(crate) fn quote_body(pieces: &[Piece]) -> TokenStream {
    let mut ids = 0u32;
    quote_pieces(pieces, &mut ids)
}

fn quote_pieces(pieces: &[Piece], ids: &mut u32) -> TokenStream {
    let mut out = TokenStream::new();
    let mut batch = Vec::new();
    for piece in pieces {
        match piece {
            Piece::Stmt(stmt) => {
                out.extend(quote_batch(&batch, ids));
                batch.clear();
                out.extend(stmt.clone());
            }
            Piece::Repeat { binding, seq, body } if tree_has_stmt(body) => {
                out.extend(quote_batch(&batch, ids));
                batch.clear();
                let seq_binding = format_ident!("__seq{}", fresh(ids));
                let inner = quote_pieces(body, ids);
                out.extend(quote! {{
                    let #seq_binding = #seq;
                    for #binding in #seq_binding {
                        #inner
                    }
                }});
            }
            _ => batch.push(piece),
        }
    }
    out.extend(quote_batch(&batch, ids));
    out
}

fn tree_has_stmt(pieces: &[Piece]) -> bool {
    pieces.iter().any(|piece| match piece {
        Piece::Stmt(_) => true,
        Piece::Repeat { body, .. } => tree_has_stmt(body),
        _ => false,
    })
}

fn quote_batch(pieces: &[&Piece], ids: &mut u32) -> TokenStream {
    let mut static_len = 0usize;
    let mut pre = TokenStream::new();
    let mut len_stmts = TokenStream::new();
    let mut writes = TokenStream::new();
    for piece in pieces {
        match piece {
            Piece::Static(text) => {
                if text.is_empty() {
                    continue;
                }
                static_len += text.len();
                writes.extend(quote! {
                    ::vize_l0::ts_emit::TsBuf::push_ts_str(__ts, #text);
                });
            }
            Piece::Hole(expr) => {
                let binding = format_ident!("__h{}", fresh(ids));
                pre.extend(quote! { let #binding = &(#expr); });
                len_stmts.extend(quote! {
                    __len += ::vize_l0::ts_emit::hole_len(#binding);
                });
                writes.extend(quote! {
                    ::vize_l0::ts_emit::write_hole(__ts, #binding);
                });
            }
            Piece::Display(expr) => {
                writes.extend(quote! {
                    ::vize_l0::ts_emit::write_display(__ts, &(#expr));
                });
            }
            Piece::Repeat { binding, seq, body } => {
                let seq_binding = format_ident!("__seq{}", fresh(ids));
                let hole_acc = format_ident!("__repeat_holes");
                let (body_static, body_len, body_write) = plan_repeat_body(body, &hole_acc, ids);
                pre.extend(quote! { let #seq_binding = #seq; });
                len_stmts.extend(quote! {
                    {
                        let mut __repeat_count: usize = 0;
                        let mut #hole_acc: usize = 0;
                        for #binding in #seq_binding.clone() {
                            __repeat_count += 1;
                            #body_len
                        }
                        __len += __repeat_count * #body_static + #hole_acc;
                    }
                });
                writes.extend(quote! {
                    for #binding in #seq_binding {
                        #body_write
                    }
                });
            }
            Piece::Stmt(_) => {}
        }
    }
    if pre.is_empty() && len_stmts.is_empty() && writes.is_empty() {
        return TokenStream::new();
    }
    let len_binding = if len_stmts.is_empty() {
        quote! { let __len: usize = #static_len; }
    } else {
        quote! {
            let mut __len: usize = #static_len;
            #len_stmts
        }
    };
    quote! {{
        #pre
        #len_binding
        ::vize_l0::ts_emit::TsBuf::reserve_ts(__ts, __len);
        #writes
    }}
}

fn plan_repeat_body(
    pieces: &[Piece],
    holes: &Ident,
    ids: &mut u32,
) -> (usize, TokenStream, TokenStream) {
    let mut static_len = 0usize;
    let mut len_stmts = TokenStream::new();
    let mut writes = TokenStream::new();
    for piece in pieces {
        match piece {
            Piece::Static(text) => {
                if text.is_empty() {
                    continue;
                }
                static_len += text.len();
                writes.extend(quote! {
                    ::vize_l0::ts_emit::TsBuf::push_ts_str(__ts, #text);
                });
            }
            Piece::Hole(expr) => {
                len_stmts.extend(quote! {
                    #holes += ::vize_l0::ts_emit::hole_len(&(#expr));
                });
                writes.extend(quote! {
                    ::vize_l0::ts_emit::write_hole(__ts, &(#expr));
                });
            }
            Piece::Display(expr) => {
                writes.extend(quote! {
                    ::vize_l0::ts_emit::write_display(__ts, &(#expr));
                });
            }
            Piece::Repeat { binding, seq, body } => {
                let seq_binding = format_ident!("__seq{}", fresh(ids));
                let inner_holes = format_ident!("__holes{}", fresh(ids));
                let (body_static, body_len, body_write) = plan_repeat_body(body, &inner_holes, ids);
                len_stmts.extend(quote! {
                    {
                        let #seq_binding = #seq;
                        let mut __repeat_count: usize = 0;
                        let mut #inner_holes: usize = 0;
                        for #binding in #seq_binding.clone() {
                            __repeat_count += 1;
                            #body_len
                        }
                        #holes += __repeat_count * #body_static + #inner_holes;
                    }
                });
                writes.extend(quote! {{
                    let #seq_binding = #seq;
                    for #binding in #seq_binding {
                        #body_write
                    }
                }});
            }
            Piece::Stmt(_) => {}
        }
    }
    (static_len, len_stmts, writes)
}

fn fresh(ids: &mut u32) -> u32 {
    let id = *ids;
    *ids += 1;
    id
}
