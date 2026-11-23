//! Check the formatter's acceptance kernel against a multiset specification.
include!("header.rs");

// The production policy reads only this ordered view of original attribute spellings.
struct StyleView<'view, 'source>(&'view [Spelling<'source>]);

impl<'view, 'source> StyleView<'view, 'source> {
    fn attrs(&self) -> &'view [Spelling<'source>] {
        self.0
    }
}

struct Spelling<'a> {
    name: &'a str,
    value: Option<&'a str>,
}

#[derive(Clone, Copy, PartialEq, Eq, kani::Arbitrary)]
enum Name {
    Lang,
    Scoped,
    Module,
    Other,
    Dynamic,
}

#[derive(Clone, Copy, PartialEq, Eq, kani::Arbitrary)]
enum Value {
    Absent,
    Empty,
    Literal,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, kani::Arbitrary)]
struct Attribute {
    name: Name,
    value: Value,
}

fn spelling(attr: Attribute) -> Spelling<'static> {
    let name = match attr.name {
        Name::Lang => "lang",
        Name::Scoped => "scoped",
        Name::Module => "module",
        Name::Other => "custom",
        Name::Dynamic => ":lang",
    };
    let value = match attr.value {
        Value::Absent => None,
        Value::Empty => Some(""),
        Value::Literal => Some("named"),
        Value::Other => Some("雪"),
    };
    Spelling { name, value }
}

#[kani::proof]
#[kani::unwind(10)]
fn accepted_headers_keep_order_or_permute_unique_plain_flags_with_their_values() {
    let before: [Attribute; 4] = kani::any();
    let after: [Attribute; 4] = kani::any();
    let n: usize = kani::any();
    let m: usize = kani::any();
    kani::assume(n <= before.len() && m <= after.len());
    let a = &before[..n];
    let b = &after[..m];
    let permutable = a.iter().all(|attr| {
        matches!(attr.name, Name::Lang | Name::Scoped | Name::Module)
            && a.iter().filter(|other| other.name == attr.name).count() == 1
    });
    let same_multiset = a.len() == b.len()
        && a.iter()
            .all(|attr| b.iter().filter(|other| *other == attr).count() == 1);
    let left = before.map(spelling);
    let right = after.map(spelling);
    assert_eq!(
        same_header(StyleView(&left[..n]), StyleView(&right[..m])),
        a == b || (permutable && same_multiset)
    );
}
