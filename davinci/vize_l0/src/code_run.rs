//! The end of a run of ordinary code bytes in generated TypeScript, sixteen
//! bytes per step where the target has 128-bit SIMD.
//!
//! The content mapper's check-mode trivia scan (`vize content-mapper`,
//! `dispatch/trivia_scan.rs`) spends most of its time stepping over code
//! between the bytes it must look at. [`skip_code`] finds the next such byte
//! with NEON (aarch64) or SSE2 (x86_64, part of every x86_64 target) and
//! falls back to [`skip_code_scalar`] elsewhere and for the last bytes. Both
//! return the same index for every input; the differential tests below run
//! the vector path against the scalar one, under Miri as well.

#![deny(clippy::undocumented_unsafe_blocks)]

/// Trivia whitespace: a run of it is handed out by the scan.
fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

/// Bytes the scan always stops at: slashes, quotes, template ticks, braces.
fn is_stop(byte: u8) -> bool {
    matches!(byte, b'/' | b'"' | b'\'' | b'`' | b'{' | b'}')
}

/// What a byte is to [`skip_code_scalar`].
const PLAIN: u8 = 0;
const SPACE: u8 = 1;
const STOP: u8 = 2;

/// Each byte's class. A table, as a `match` per byte costs the scalar scan
/// about 40% more instructions.
const CLASSES: [u8; 256] = {
    let mut classes = [PLAIN; 256];
    let mut byte = 0;
    while byte < classes.len() {
        if let Some((_, rest)) = classes.split_at_mut_checked(byte)
            && let Some(slot) = rest.first_mut()
        {
            *slot = match byte as u8 {
                b' ' | b'\t' | b'\n' | b'\r' => SPACE,
                b'/' | b'"' | b'\'' | b'`' | b'{' | b'}' => STOP,
                _ => PLAIN,
            };
        }
        byte += 1;
    }
    classes
};

fn class(byte: u8) -> u8 {
    CLASSES.get(usize::from(byte)).copied().unwrap_or(PLAIN)
}

/// The first index from `at` that ends a run of code: a stop byte
/// (`/ " ' \` { }`), whitespace followed by whitespace or `/`, whitespace at
/// the end, or the end. A lone whitespace byte between two code bytes is part
/// of the run. Every other byte (`<` included) is code.
pub fn skip_code(bytes: &[u8], at: usize) -> usize {
    #[cfg(any(
        all(target_arch = "aarch64", target_feature = "neon"),
        all(target_arch = "x86_64", target_feature = "sse2"),
    ))]
    {
        let mut at = at;
        // Each window holds sixteen candidate bytes and the byte after the
        // last one, which decides whether a whitespace candidate ends the run.
        while let Some(window) = bytes.get(at..).and_then(<[u8]>::first_chunk::<17>) {
            // SAFETY: `run_ends` only needs its target feature, and the cfg
            // on this block compiles it only for targets where that feature
            // (NEON on aarch64, SSE2 on x86_64) is enabled for the whole
            // program.
            let ends = unsafe { vector::run_ends(window) };
            if ends != 0 {
                return at + vector::first(ends);
            }
            at += 16;
        }
        skip_code_scalar(bytes, at)
    }
    #[cfg(not(any(
        all(target_arch = "aarch64", target_feature = "neon"),
        all(target_arch = "x86_64", target_feature = "sse2"),
    )))]
    skip_code_scalar(bytes, at)
}

/// [`skip_code`], one byte at a time.
pub fn skip_code_scalar(bytes: &[u8], mut at: usize) -> usize {
    loop {
        while bytes.get(at).is_some_and(|byte| class(*byte) == PLAIN) {
            at += 1;
        }
        match (bytes.get(at), bytes.get(at + 1)) {
            (Some(byte), Some(next))
                if class(*byte) == SPACE && class(*next) != SPACE && *next != b'/' =>
            {
                at += 1;
            }
            _ => return at,
        }
    }
}

/// Which of a window's first sixteen bytes end a run, as `is_stop(byte) ||
/// is_space(byte) && (is_space(next) || next == b'/')` per lane.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod vector {
    use core::arch::aarch64::{
        uint8x16_t, vandq_u8, vceqq_u8, vdupq_n_u8, vget_lane_u64, vld1q_u8, vorrq_u8,
        vreinterpret_u64_u8, vreinterpretq_u16_u8, vshrn_n_u16,
    };

    #[target_feature(enable = "neon")]
    fn eq(lanes: uint8x16_t, byte: u8) -> uint8x16_t {
        vceqq_u8(lanes, vdupq_n_u8(byte))
    }

    #[target_feature(enable = "neon")]
    fn space(lanes: uint8x16_t) -> uint8x16_t {
        vorrq_u8(
            vorrq_u8(eq(lanes, b' '), eq(lanes, b'\t')),
            vorrq_u8(eq(lanes, b'\n'), eq(lanes, b'\r')),
        )
    }

    #[target_feature(enable = "neon")]
    fn stop(lanes: uint8x16_t) -> uint8x16_t {
        vorrq_u8(
            vorrq_u8(
                vorrq_u8(eq(lanes, b'/'), eq(lanes, b'"')),
                vorrq_u8(eq(lanes, b'\''), eq(lanes, b'`')),
            ),
            vorrq_u8(eq(lanes, b'{'), eq(lanes, b'}')),
        )
    }

    /// Four bits per lane, lane `i` at bits `4 * i..4 * i + 4`; zero when no
    /// lane ends the run.
    #[target_feature(enable = "neon")]
    pub(super) fn run_ends(window: &[u8; 17]) -> u64 {
        // SAFETY: `window` is 17 initialized bytes, so the 16 bytes read at
        // offset 0 and the 16 bytes read at offset 1 both lie inside it;
        // `vld1q_u8` has no alignment requirement, and this function enables
        // NEON.
        let (current, next) = unsafe {
            let at = window.as_ptr();
            (vld1q_u8(at), vld1q_u8(at.add(1)))
        };
        let ends = vorrq_u8(
            stop(current),
            vandq_u8(space(current), vorrq_u8(space(next), eq(next, b'/'))),
        );
        // Narrow each 0x00/0xFF lane to a nibble: the u16 lane holding lanes
        // 2k and 2k+1, shifted right by 4, keeps the high nibble of lane 2k
        // and the low nibble of lane 2k+1 in its low byte.
        vget_lane_u64::<0>(vreinterpret_u64_u8(vshrn_n_u16::<4>(vreinterpretq_u16_u8(
            ends,
        ))))
    }

    /// The first lane set in [`run_ends`]'s non-zero result.
    pub(super) fn first(ends: u64) -> usize {
        (ends.trailing_zeros() / 4) as usize
    }
}

/// Which of a window's first sixteen bytes end a run, as `is_stop(byte) ||
/// is_space(byte) && (is_space(next) || next == b'/')` per lane.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
mod vector {
    use core::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128,
        _mm_set1_epi8,
    };

    #[target_feature(enable = "sse2")]
    fn eq(lanes: __m128i, byte: u8) -> __m128i {
        _mm_cmpeq_epi8(lanes, _mm_set1_epi8(byte as i8))
    }

    #[target_feature(enable = "sse2")]
    fn space(lanes: __m128i) -> __m128i {
        _mm_or_si128(
            _mm_or_si128(eq(lanes, b' '), eq(lanes, b'\t')),
            _mm_or_si128(eq(lanes, b'\n'), eq(lanes, b'\r')),
        )
    }

    #[target_feature(enable = "sse2")]
    fn stop(lanes: __m128i) -> __m128i {
        _mm_or_si128(
            _mm_or_si128(
                _mm_or_si128(eq(lanes, b'/'), eq(lanes, b'"')),
                _mm_or_si128(eq(lanes, b'\''), eq(lanes, b'`')),
            ),
            _mm_or_si128(eq(lanes, b'{'), eq(lanes, b'}')),
        )
    }

    /// One bit per lane; zero when no lane ends the run.
    #[target_feature(enable = "sse2")]
    pub(super) fn run_ends(window: &[u8; 17]) -> u64 {
        // SAFETY: `window` is 17 initialized bytes, so the 16 bytes read at
        // offset 0 and the 16 bytes read at offset 1 both lie inside it;
        // `_mm_loadu_si128` has no alignment requirement, and this function
        // enables SSE2.
        let (current, next) = unsafe {
            let at = window.as_ptr();
            (
                _mm_loadu_si128(at.cast::<__m128i>()),
                _mm_loadu_si128(at.add(1).cast::<__m128i>()),
            )
        };
        let ends = _mm_or_si128(
            stop(current),
            _mm_and_si128(space(current), _mm_or_si128(space(next), eq(next, b'/'))),
        );
        u64::from(_mm_movemask_epi8(ends).cast_unsigned())
    }

    /// The first lane set in [`run_ends`]'s non-zero result.
    pub(super) fn first(ends: u64) -> usize {
        ends.trailing_zeros() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{is_space, is_stop, skip_code, skip_code_scalar};

    /// Fewer rounds under Miri, which interprets every intrinsic.
    const ROUNDS: usize = if cfg!(miri) { 300 } else { 20_000 };

    /// Bytes the scan treats differently, beside ordinary code and non-ASCII.
    const ALPHABET: &[u8] = b"  \t\n\r/\"'`{}<>a_0;.*\\$\x00\x7f\xe2\x80\xa8\xff";

    #[test]
    fn every_byte_is_classified_as_the_scan_rules_say() {
        for byte in 0..=u8::MAX {
            assert_eq!(
                super::class(byte),
                if is_space(byte) {
                    super::SPACE
                } else if is_stop(byte) {
                    super::STOP
                } else {
                    super::PLAIN
                },
                "{byte:#04x}"
            );
        }
    }

    /// Every (byte, next byte) pair at every lane of a window, the lane after
    /// the last included: the vector path returns the scalar path's index.
    #[test]
    fn every_byte_pair_at_every_lane_matches_the_scalar_scan() {
        let tried: Vec<u8> = if cfg!(miri) {
            ALPHABET.to_vec()
        } else {
            (0..=u8::MAX).collect()
        };
        for &byte in &tried {
            for &next in &tried {
                for lane in 0..17 {
                    let mut bytes = [b'a'; 18];
                    bytes[lane] = byte;
                    bytes[lane + 1] = next;
                    assert_eq!(
                        skip_code(&bytes, 0),
                        skip_code_scalar(&bytes, 0),
                        "{byte:#04x} {next:#04x} at lane {lane}"
                    );
                }
            }
        }
    }

    /// Random soups of the bytes the scan cares about, at every start offset:
    /// the vector path returns the scalar path's index.
    #[test]
    fn the_vector_scan_matches_the_scalar_scan() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = |bound: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % bound as u64) as usize
        };
        for round in 0..ROUNDS {
            let len = next(80);
            // Long plain runs as well as dense specials.
            let density = [2, 6, 30][next(3)];
            let bytes = (0..len)
                .map(|_| {
                    if next(density) == 0 {
                        ALPHABET[next(ALPHABET.len())]
                    } else {
                        b"abcdefghij0123=(),.:"[next(20)]
                    }
                })
                .collect::<Vec<u8>>();
            for at in 0..=len + 1 {
                assert_eq!(
                    skip_code(&bytes, at),
                    skip_code_scalar(&bytes, at),
                    "round {round}, at {at}: {:?}",
                    String::from_utf8_lossy(&bytes)
                );
            }
        }
    }

    #[test]
    fn a_run_ends_at_stops_and_at_whitespace_before_whitespace_or_a_slash() {
        let text = b"const value = call(alpha, beta) + gamma[0];  // note";
        // Lone spaces are crossed; the double space before `//` ends the run.
        assert_eq!(skip_code(text, 0), 43);
        assert_eq!(skip_code(b"a b", 0), 3);
        assert_eq!(skip_code(b"a /", 0), 1);
        assert_eq!(skip_code(b"a ", 0), 1);
        assert_eq!(skip_code(b"a<b {", 0), 4);
        assert_eq!(skip_code(b"", 0), 0);
        assert_eq!(skip_code(b"abc", 7), 7);
    }
}
