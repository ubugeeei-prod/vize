//! Exact empty-assignment corrections to immutable tokenizer history.
//!
//! The binary retains authentic whole BEFORE/AFTER trees and errors from
//! Actions 39119566648. Its adjacent manifest binds every source, index and
//! digest. This never executes current compatibility code as historical code,
//! projects actual trees, or admits cases outside the declared 66 contexts.

use super::tokenizer_first_newline_witness;
use vize_l0::{config::VueVersion, cstr, hash::StableHasher128};
use vize_relief::options::{ParseMode, ParserOptions, WhitespaceStrategy};

const CAPTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/_fixtures/empty_attribute_value_witness.bin"
));
const INDICES: [usize; 33] = [
    19, 21, 47, 49, 81, 83, 161, 163, 203, 205, 300, 419, 470, 482, 494, 567, 569, 589, 591, 823,
    880, 959, 995, 1349, 1351, 1379, 1530, 1549, 1672, 1674, 1771, 1781, 2082,
];
const PROFILE_OFFSET: usize = 2148;
pub(super) const ALL_HITS: u128 = (1_u128 << 66) - 1;
pub(super) const AFTER_AGGREGATE: [u8; 16] = [
    67, 153, 214, 222, 51, 158, 150, 212, 169, 35, 28, 140, 178, 190, 238, 27,
];

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn integer(&mut self) -> usize {
        let bytes = self.0.get(..8).expect("complete witness integer");
        let value = u64::from_le_bytes(bytes.try_into().expect("eight bytes"));
        self.0 = self.0.get(8..).expect("integer remainder");
        value.try_into().expect("witness length fits the host")
    }

    fn field(&mut self) -> &'a [u8] {
        let length = self.integer();
        let value = self.0.get(..length).expect("complete witness field");
        self.0 = self.0.get(length..).expect("field remainder");
        value
    }

    fn text(&mut self) -> &'a str {
        core::str::from_utf8(self.field()).expect("authored UTF-8 witness")
    }
}

#[derive(Clone, Copy)]
struct Witness<'a> {
    context: &'a str,
    source: &'a str,
    before: &'a str,
    after: &'a str,
    errors: &'a str,
    before_digest: &'a [u8],
    after_digest: &'a [u8],
}

fn witnesses() -> [Witness<'static>; 33] {
    let magic = b"VIZE_EMPTY_VALUE_WITNESS_V1\0";
    assert_eq!(CAPTURE.get(..magic.len()), Some(magic.as_slice()));
    let mut reader = Reader(CAPTURE.get(magic.len()..).expect("witness header"));
    for required in [
        "54400c86c8a224777640d9554c220dccf8f53b90",
        "bf006b1c11996197296c4344eeb9153a6289fdad",
        "b8faa65620b1833b457d7ef24cc24956b6084b4c095df1fe6834f9c8e50059be",
    ] {
        assert_eq!(reader.text(), required, "exact captured source authority");
    }
    assert_eq!(
        reader.field(),
        tokenizer_first_newline_witness::APPROVED_AGGREGATE.as_slice()
    );
    assert_eq!(reader.field(), AFTER_AGGREGATE.as_slice());
    assert_eq!(
        reader.text(),
        "8d4d06bffecd8503651553d312e444c5ba2640db98af592b8224517ff0de44a7"
    );
    assert_eq!(
        reader.text(),
        "ac74b09895e213f82286d94b7eb280ec8e7de3b9e03da3d54033b2913f0f5c42"
    );
    assert_eq!(reader.integer(), INDICES.len());
    let entries = core::array::from_fn(|position| {
        assert_eq!(
            reader.integer(),
            *INDICES.get(position).expect("declared index")
        );
        let entry = Witness {
            context: reader.text(),
            source: reader.text(),
            before: reader.text(),
            after: reader.text(),
            errors: reader.text(),
            before_digest: reader.field(),
            after_digest: reader.field(),
        };
        assert_eq!(entry.before_digest.len(), 16);
        assert_eq!(entry.after_digest.len(), 16);
        entry
    });
    assert!(reader.0.is_empty(), "no trailing witness bytes");
    entries
}

fn digest(source: &[u8], tree: &[u8], errors: &[u8]) -> [u8; 16] {
    let mut hash = StableHasher128::new();
    for field in [source, tree, errors] {
        hash.update(&(field.len() as u64).to_le_bytes());
        hash.update(field);
    }
    hash.digest()
}

fn default_fixture_options(options: &ParserOptions) -> bool {
    let ParserOptions {
        mode,
        whitespace,
        delimiters,
        is_pre_tag,
        is_native_tag,
        is_custom_element,
        custom_renderer,
        is_void_tag,
        get_namespace,
        on_error,
        on_warn,
        comments,
        experimental_in_tag_comments,
        dialect,
    } = options;
    let defaults = ParserOptions::default();
    *mode == ParseMode::Base
        && *whitespace == WhitespaceStrategy::Condense
        && delimiters == &defaults.delimiters
        && core::ptr::fn_addr_eq(*is_pre_tag, defaults.is_pre_tag)
        && is_native_tag.is_none()
        && is_custom_element.is_none()
        && !*custom_renderer
        && core::ptr::fn_addr_eq(*is_void_tag, defaults.is_void_tag)
        && core::ptr::fn_addr_eq(*get_namespace, defaults.get_namespace)
        && on_error.is_none()
        && on_warn.is_none()
        && *comments
        && !*experimental_in_tag_comments
        && *dialect == VueVersion::V3
}

pub(super) fn corrected_digest(
    index: usize,
    fields: &[&[u8]; 3],
    context: &str,
    options: &ParserOptions,
    historical: Option<&[u8]>,
) -> Option<([u8; 16], u128)> {
    let (position, document) = INDICES.iter().enumerate().find_map(|(position, base)| {
        (index == *base)
            .then_some((position, false))
            .or_else(|| (index == *base + PROFILE_OFFSET).then_some((position, true)))
    })?;
    let entry = *witnesses().get(position)?;
    let [source, tree, errors] = fields;
    let expected_context = cstr!("document={document}; {}", entry.context);
    (context == expected_context.as_str()
        && default_fixture_options(options)
        && *source == entry.source.as_bytes()
        && *tree == entry.after.as_bytes()
        && *errors == entry.errors.as_bytes()
        && historical == Some(entry.before_digest)
        && digest(source, entry.before.as_bytes(), errors).as_slice() == entry.before_digest
        && digest(source, tree, errors).as_slice() == entry.after_digest)
        .then(|| {
            let after: [u8; 16] = entry
                .after_digest
                .try_into()
                .expect("complete AFTER digest");
            let slot = position + if document { INDICES.len() } else { 0 };
            (after, 1_u128 << slot)
        })
}

#[test]
fn all_captured_preimages_keep_original_hashes_and_exact_successor_bindings() {
    let frozen = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/_fixtures/tokenizer_parser_fixtures.bin"
    ));
    assert_eq!(frozen.len(), (4230 + 66) * 16);
    let options = ParserOptions::default();
    let mut hits = 0;
    for (position, entry) in witnesses().iter().enumerate() {
        let base = *INDICES.get(position).expect("declared base");
        for (document, index) in [(false, base), (true, base + PROFILE_OFFSET)] {
            let context = cstr!("document={document}; {}", entry.context);
            let original = frozen.get(index * 16..index * 16 + 16);
            assert_eq!(original, Some(entry.before_digest));
            let fields = [
                entry.source.as_bytes(),
                entry.after.as_bytes(),
                entry.errors.as_bytes(),
            ];
            let (actual, bit) = corrected_digest(index, &fields, &context, &options, original)
                .expect("whole authentic successor");
            assert_eq!(actual.as_slice(), entry.after_digest);
            assert_eq!(hits & bit, 0, "each exact context participates once");
            hits |= bit;
        }
    }
    assert_eq!(hits, ALL_HITS);
}

#[test]
fn historical_authorization_refuses_wrong_identity_options_and_whole_packet_fields() {
    for (position, entry) in witnesses().iter().enumerate() {
        let index = *INDICES.get(position).expect("declared base");
        let context = cstr!("document=false; {}", entry.context);
        let options = ParserOptions::default();
        let fields = [
            entry.source.as_bytes(),
            entry.after.as_bytes(),
            entry.errors.as_bytes(),
        ];
        let accepts = |index: usize,
                       fields: &[&[u8]; 3],
                       context: &str,
                       options: &ParserOptions,
                       historical: Option<&[u8]>| {
            corrected_digest(index, fields, context, options, historical).is_some()
        };
        assert!(!accepts(
            0,
            &fields,
            &context,
            &options,
            Some(entry.before_digest)
        ));
        assert!(!accepts(
            index + PROFILE_OFFSET,
            &fields,
            &context,
            &options,
            Some(entry.before_digest)
        ));
        assert!(!accepts(
            index,
            &fields,
            "foreign context",
            &options,
            Some(entry.before_digest)
        ));
        assert!(!accepts(
            index,
            &fields,
            &context,
            &options,
            Some(entry.after_digest)
        ));
        assert!(!accepts(index, &fields, &context, &options, None));
        let mut changed_options = options.clone();
        changed_options.comments = false;
        assert!(!accepts(
            index,
            &fields,
            &context,
            &changed_options,
            Some(entry.before_digest)
        ));
        for field in 0..3 {
            let mut changed = fields;
            let mut bytes = fields.get(field).expect("field").to_vec();
            bytes.push(b'!');
            *changed.get_mut(field).expect("field") = &bytes;
            assert!(!accepts(
                index,
                &changed,
                &context,
                &options,
                Some(entry.before_digest)
            ));
        }
        let old = [
            entry.source.as_bytes(),
            entry.before.as_bytes(),
            entry.errors.as_bytes(),
        ];
        assert!(!accepts(
            index,
            &old,
            &context,
            &options,
            Some(entry.before_digest)
        ));
        for (before, after) in [
            ("content: \"\"", "content: \"fabricated\""),
            ("shorthand: false", "shorthand: true"),
            ("js_ast: None", "js_ast: Some(fabricated)"),
            ("start: ", "start: 9"),
        ] {
            if entry.after.contains(before) {
                let mutated = entry.after.replacen(before, after, 1);
                let changed = [
                    entry.source.as_bytes(),
                    mutated.as_bytes(),
                    entry.errors.as_bytes(),
                ];
                assert!(!accepts(
                    index,
                    &changed,
                    &context,
                    &options,
                    Some(entry.before_digest)
                ));
            }
        }
    }
}
