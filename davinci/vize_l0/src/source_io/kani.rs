//! Kani ties the Table 3-7 transcription in `utf8.rs` to the standard
//! library's decoder: on every byte sequence of up to four bytes, the longest
//! scalar encoding, `is_well_formed` accepts exactly what
//! `core::str::from_utf8` accepts. Creusot proves `is_well_formed` decides
//! the logic predicate the composition laws are stated over. This file is a
//! standalone Kani root (`kani davinci/vize_l0/src/source_io/kani.rs`); no
//! product build includes it or `utf8.rs`.

mod utf8;

#[cfg(kani)]
mod proofs {
    use super::utf8::is_well_formed;

    #[kani::proof]
    #[kani::unwind(6)]
    fn the_well_formed_check_agrees_with_the_standard_decoder() {
        let bytes: [u8; 4] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= bytes.len());
        let prefix = bytes.get(..len).unwrap_or_default();
        let expected = core::str::from_utf8(prefix).is_ok();
        assert_eq!(is_well_formed(prefix), expected);
        kani::cover!(expected && len == 4, "a four-byte scalar is accepted");
        kani::cover!(
            expected && len == 3 && prefix.first() == Some(&0xED),
            "a non-surrogate ED row is accepted"
        );
        kani::cover!(
            !expected && prefix.first() == Some(&0xED),
            "a surrogate is rejected"
        );
        kani::cover!(
            !expected && prefix.first() == Some(&0xC0),
            "an overlong lead is rejected"
        );
        kani::cover!(expected && len == 0, "the empty sequence is accepted");
    }
}
