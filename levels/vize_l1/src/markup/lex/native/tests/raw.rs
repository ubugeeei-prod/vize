use super::{LexOptions, TestCallbacks, TokenEvent, lex_with, tokenize};

// Vue 1.x triple-mustache raw-HTML interpolation.

/// Without `raw_interpolation`, `{{{ x }}}` tokenizes exactly as today: a
/// `{{ … }}` interpolation whose expression keeps the leading `{`, plus a
/// trailing `}` text node. This is the zero-cost default path.

#[test]
fn triple_mustache_default_is_braced_interpolation_plus_text() {
    // "{{{ x }}}": indices 0..9
    let cb = tokenize("{{{ x }}}");
    assert!(cb.errors.is_empty());
    // Expression span is `{ x ` -> [2, 6); trailing `}` text is [8, 9).
    assert!(cb.events.contains(&TokenEvent::Interpolation(2, 6)));
    assert!(cb.events.contains(&TokenEvent::Text(8, 9)));
    // With no capability set, the raw-interpolation callback never fires.
    assert!(
        !cb.events
            .iter()
            .any(|e| matches!(e, TokenEvent::RawInterpolation(..))),
        "no raw interpolation without the capability"
    );
}

fn tokenize_triple(input: &str) -> TestCallbacks {
    let options = LexOptions {
        raw_interpolation: true,
        ..LexOptions::default()
    };
    lex_with(input, options)
}

#[test]
fn triple_mustache_with_capability_emits_raw_interpolation() {
    // "{{{ x }}}": the expression span is ` x ` -> [3, 6); both extra braces
    // are dropped and no trailing text is produced.
    let cb = tokenize_triple("{{{ x }}}");
    assert!(cb.errors.is_empty(), "{:?}", cb.errors);
    assert!(cb.events.contains(&TokenEvent::RawInterpolation(3, 6)));
    assert!(
        !cb.events.iter().any(|e| matches!(e, TokenEvent::Text(..))),
        "no stray brace text: {:?}",
        cb.events
    );
}

#[test]
fn double_mustache_with_capability_is_unchanged() {
    // A plain `{{ x }}` is still an ordinary (escaped) interpolation even when
    // triple-mustache recognition is enabled.
    let cb = tokenize_triple("{{ x }}");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::Interpolation(2, 5)));
    assert!(
        !cb.events
            .iter()
            .any(|e| matches!(e, TokenEvent::RawInterpolation(..))),
        "`{{{{ … }}}}` stays a plain interpolation"
    );
}

#[test]
fn triple_mustache_adjacent_and_mixed_with_text() {
    let cb = tokenize_triple("a {{{ x }}} b");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::Text(0, 2))); // "a "
    assert!(cb.events.contains(&TokenEvent::RawInterpolation(5, 8))); // " x "
    assert!(cb.events.contains(&TokenEvent::Text(11, 13))); // " b"
}
