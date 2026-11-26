//! Line breaks in the whitespace between two template chunks.
#[cfg(creusot)]
use creusot_std::prelude::*;

/// A gap of exactly one authored line break (LF, CR LF or a lone CR) and
/// otherwise only spaces and tabs, with the break first.
#[cfg_attr(creusot, ensures(result == (gap@.len() > 0 && (gap@[0]@ == 0x0A || gap@[0]@ == 0x0D)
    && blank(gap@, gap@.len()) && breaks(gap@, gap@.len()) == 1)))]
#[inline(always)]
pub(super) fn is_single_line_break_gap(gap: &[u8]) -> bool {
    let mut index: usize = match gap.first() {
        Some(b'\n') => 1,
        Some(b'\r') if gap.get(1) == Some(&b'\n') => 2,
        Some(b'\r') => 1,
        _ => return false,
    };
    #[cfg_attr(creusot, invariant(1 <= index@ && index@ <= gap@.len()))]
    #[cfg_attr(creusot, invariant(blank(gap@, index@) && breaks(gap@, index@) == 1))]
    #[cfg_attr(creusot, invariant(index@ < gap@.len() ==> gap@[index@ - 1]@ != 0x0D || gap@[index@]@ != 0x0A))]
    #[cfg_attr(creusot, variant(gap@.len() - index@))]
    while let Some(&byte) = gap.get(index) {
        if !matches!(byte, b' ' | b'\t') {
            #[cfg(creusot)]
            let _ = snapshot!(breaks_never_decrease(gap@, index@ + 1, gap@.len()));
            return false;
        }
        index += 1;
    }
    true
}

/// Count LF, CRLF or lone-CR boundaries in an all-whitespace source gap.
///
/// `None` when a byte is not HTML whitespace as the template formatter reads
/// it (space, tab, LF, CR); otherwise the number of line breaks after the
/// input stream's newline normalization (a CR LF pair is one), saturated at
/// `u8::MAX`.
#[cfg_attr(creusot, ensures(match result {
    None => !blank(gap@, gap@.len()),
    Some(count) => blank(gap@, gap@.len())
        && count@ == if breaks(gap@, gap@.len()) < 255 { breaks(gap@, gap@.len()) } else { 255 },
}))]
#[inline(always)]
pub(super) fn whitespace_gap_line_breaks(gap: &[u8]) -> Option<u8> {
    let mut line_breaks = 0_u8;
    let mut index: usize = 0;
    #[cfg_attr(creusot, invariant(index@ <= gap@.len()))]
    #[cfg_attr(creusot, invariant(blank(gap@, index@)))]
    #[cfg_attr(creusot, invariant(line_breaks@ == if breaks(gap@, index@) < 255 { breaks(gap@, index@) } else { 255 }))]
    #[cfg_attr(creusot, invariant(0 < index@ && index@ < gap@.len() ==> gap@[index@ - 1]@ != 0x0D || gap@[index@]@ != 0x0A))]
    #[cfg_attr(creusot, variant(gap@.len() - index@))]
    while let Some(&byte) = gap.get(index) {
        match byte {
            b'\r' => {
                line_breaks = line_breaks.saturating_add(1);
                if gap.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
            }
            b'\n' => line_breaks = line_breaks.saturating_add(1),
            byte if !matches!(byte, b' ' | b'\t' | b'\n' | b'\r') => return None,
            _ => {}
        }
        index += 1;
    }
    Some(line_breaks)
}

/// The first `len` bytes are space, tab, LF or CR.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn blank(gap: Seq<u8>, len: Int) -> bool {
    pearlite! { forall<i: Int> 0 <= i && i < len
    ==> gap[i]@ == 0x20 || gap[i]@ == 0x09 || gap[i]@ == 0x0A || gap[i]@ == 0x0D }
}

/// The line breaks in the first `len` bytes after the HTML input stream
/// normalizes newlines: every CR is one, and so is every LF that does not
/// follow a CR.
#[cfg(creusot)]
#[logic(open(crate))]
#[variant(len)]
pub(crate) fn breaks(gap: Seq<u8>, len: Int) -> Int {
    pearlite! {
        if len <= 0 {
            0
        } else if gap[len - 1]@ == 0x0D || (gap[len - 1]@ == 0x0A && !(len >= 2 && gap[len - 2]@ == 0x0D)) {
            breaks(gap, len - 1) + 1
        } else {
            breaks(gap, len - 1)
        }
    }
}

/// A longer prefix holds at least as many line breaks.
#[cfg(creusot)]
#[logic]
#[variant(j - i)]
#[requires(0 <= i && i <= j && j <= gap.len())]
#[ensures(breaks(gap, i) <= breaks(gap, j))]
pub(crate) fn breaks_never_decrease(gap: Seq<u8>, i: Int, j: Int) {
    pearlite! {
        if i < j {
            breaks_never_decrease(gap, i, j - 1)
        }
    }
}
