use super::{Cell, Probe, StableHasher128, String, Tokenizer, cstr, feed, oracle};

#[test]
fn interrupted_callback_and_non_eof_profile_change_keep_the_frozen_contract() {
    let mut cases = oracle(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/_fixtures/tokenizer_callback_resume.bin"
    )));
    extern crate std;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let source = "<div id='x' // comment\n :name='y'>{{{raw}}}</div><!bogus>";
    let mut result = String::default();
    let polls = Cell::new(0);
    let pause = Cell::new(false);
    let probe = Probe {
        trace: &mut result,
        polls: &polls,
        pause: Some(&pause),
        live_mode: false,
        verbatim: false,
    };
    let mut tokenizer = Tokenizer::new(source, probe);
    assert!(catch_unwind(AssertUnwindSafe(|| tokenizer.tokenize())).is_err());
    tokenizer.set_tolerate_declarations(true);
    tokenizer.set_in_tag_comments(true);
    tokenizer.set_triple_mustache(true);
    tokenizer.tokenize();
    tokenizer.set_tolerate_declarations(false);
    tokenizer.set_in_tag_comments(false);
    tokenizer.set_triple_mustache(false);
    tokenizer.tokenize();
    drop(tokenizer);
    result.push_str(&cstr!("mode-polls:{};", polls.get()));
    let mut hash = StableHasher128::new();
    cases.check(
        &[source.as_bytes(), result.as_bytes()],
        "non-EOF callback pause and profile switch",
    );
    cases.finish();
    feed(&mut hash, source.as_bytes());
    feed(&mut hash, result.as_bytes());
    assert_eq!(
        hash.digest(),
        [
            158, 92, 108, 69, 251, 99, 193, 37, 48, 125, 186, 47, 216, 189, 183, 150
        ],
        "captured source: 5bca3a881"
    );
}
