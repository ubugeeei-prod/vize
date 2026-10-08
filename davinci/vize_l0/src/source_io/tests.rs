use super::decode_utf8;

fn assert_decode_matches_std(bytes: &[u8]) {
    let expected = core::str::from_utf8(bytes);
    let actual = decode_utf8(bytes);
    match (expected, actual) {
        (Ok(expected), Ok(actual)) => {
            assert_eq!(actual, expected);
            assert_eq!(actual.as_ptr(), bytes.as_ptr());
        }
        (Err(expected), Err(actual)) => {
            assert_eq!(actual.valid_up_to(), expected.valid_up_to());
            assert_eq!(actual.error_len(), expected.error_len());
            assert_eq!(actual.to_string(), expected.to_string());
        }
        _ => assert_eq!(actual, expected, "UTF-8 verdict differs"),
    }
}

#[test]
fn all_one_and_two_byte_inputs_match_the_standard_decoder() {
    for first in 0..=u8::MAX {
        assert_decode_matches_std(&[first]);
        for second in 0..=u8::MAX {
            assert_decode_matches_std(&[first, second]);
            // Place the same sequence across a SIMD block boundary as well.
            let mut bytes = vec![b'a'; 63];
            bytes.extend([first, second]);
            assert_decode_matches_std(&bytes);
        }
    }
}

#[test]
fn unicode_truncation_overlong_surrogates_and_block_boundaries_match_std() {
    let sequences: &[&[u8]] = &[
        "é日本語🦀".as_bytes(),
        &[0xc0, 0xaf],
        &[0xe0, 0x80, 0xaf],
        &[0xed, 0xa0, 0x80],
        &[0xf0, 0x80, 0x80, 0xaf],
        &[0xf4, 0x90, 0x80, 0x80],
        &[0xff],
        &[0xe2, 0x28, 0xa1],
    ];
    for padding in [0, 1, 61, 62, 63, 64, 65, 127, 255, 1023] {
        for sequence in sequences {
            for length in 0..=sequence.len() {
                let mut bytes = vec![b'a'; padding];
                bytes.extend(sequence.iter().take(length));
                assert_decode_matches_std(&bytes);
            }
        }
    }
}

#[test]
fn deterministic_random_external_buffers_match_std() {
    let mut random = 0x41c6_4e6d_u32;
    for length in 0..4096 {
        let bytes: Vec<_> = (0..length)
            .map(|_| {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                random as u8
            })
            .collect();
        assert_decode_matches_std(&bytes);
    }
}
