//! Differential between the Rust P3-2 evaluator and an independent
//! classifier written from the declarative rule table in
//! `docs/davinci/plan/phase-3.md`, never from Rust source. The committed
//! `[s3-reactivity-dump]` page is what Rust prints for an enumerated matrix;
//! this parses it independently, re-enumerates the matrix and recomputes every
//! class. The matrix is all 256 effect sets for an inert local binding, then
//! every origin and escape pair with no effect and with each single effect,
//! which reaches every combination of rule floors.

/// The value axis, most to least stable; `join` is "less stable wins".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    Static,
    PropsStable,
    Reactive,
    Unstable,
}

const CLASSES: [(Class, &str); 4] = [
    (Class::Static, "static"),
    (Class::PropsStable, "props-stable"),
    (Class::Reactive, "reactive"),
    (Class::Unstable, "unstable"),
];

fn join(a: Class, b: Class) -> Class {
    a.max(b)
}

/// Effects in the page's print order, each with the floor it demands.
const EFFECTS: [(&str, Class); 8] = [
    ("freeze", Class::PropsStable),
    ("capture", Class::Reactive),
    ("read-prop", Class::PropsStable),
    ("read-reactive", Class::Reactive),
    ("mutate-local", Class::Reactive),
    ("mutate-global", Class::Unstable),
    ("call-unknown", Class::Unstable),
    ("allocate", Class::PropsStable),
];

const ORIGINS: [(&str, Class); 4] = [
    ("local", Class::Static),
    ("prop", Class::PropsStable),
    ("provide-inject", Class::Reactive),
    ("template-ref", Class::Reactive),
];

const ESCAPES: [(&str, Class); 4] = [
    ("none", Class::Static),
    ("returned", Class::PropsStable),
    ("stored", Class::Reactive),
    ("global", Class::Unstable),
];

const VERDICTS: [&str; 3] = ["proven", "refuted", "unknown"];

/// A classifier under test.
pub type Classifier = fn(&Input) -> Class;

/// One binding summary: origin, effect and escape positions in the tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub origin: usize,
    pub effects: Vec<usize>,
    pub escape: usize,
}

fn floor(table: &[(&str, Class)], index: usize) -> Class {
    table
        .get(index)
        .map_or(Class::Unstable, |(_, class)| *class)
}

pub fn effects_floor(effects: &[usize]) -> Class {
    effects
        .iter()
        .fold(Class::Static, |acc, e| join(floor(&EFFECTS, *e), acc))
}

/// Naive evaluator: join every floor, then apply the provide/inject cap.
pub fn classify(input: &Input) -> Class {
    let value = join(
        join(floor(&ORIGINS, input.origin), effects_floor(&input.effects)),
        floor(&ESCAPES, input.escape),
    );
    match input.origin {
        2 => join(value, Class::Reactive),
        _ => value,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    id: usize,
    class: Class,
    verdict: usize,
    input: Input,
    span: (u64, u64),
}

fn spelled<T: Copy>(table: &[(T, &str)], what: &str, text: &str) -> Result<T, String> {
    match table
        .iter()
        .filter(|(_, s)| *s == text)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [(value, _)] => Ok(*value),
        _ => Err(format!("unknown {what} `{text}`")),
    }
}

fn position(names: &[&str], what: &str, text: &str) -> Result<usize, String> {
    let table: Vec<(usize, &str)> = names.iter().copied().enumerate().collect();
    spelled(&table, what, text)
}

fn names(table: &[(&'static str, Class)]) -> Vec<&'static str> {
    table.iter().map(|(name, _)| *name).collect()
}

fn field<'t>(name: &str, token: &'t str) -> Result<&'t str, String> {
    (token.strip_prefix(name))
        .and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| format!("expected `{name}=...`, got `{token}`"))
}

fn number(text: &str) -> Result<u64, String> {
    match !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
        true => text.parse().map_err(|_| format!("invalid number `{text}`")),
        false => Err(format!("invalid number `{text}`")),
    }
}

fn parse_effects(text: &str) -> Result<Vec<usize>, String> {
    if text == "-" {
        return Ok(Vec::new());
    }
    let effects = (text.split(','))
        .map(|e| position(&names(&EFFECTS), "effect", e))
        .collect::<Result<Vec<_>, _>>()?;
    let canonical: Vec<usize> = (0..EFFECTS.len()).filter(|e| effects.contains(e)).collect();
    if canonical != effects {
        return Err(format!(
            "effects `{text}` must be distinct and in vocabulary order"
        ));
    }
    Ok(effects)
}

fn parse_row(line: &str) -> Result<Row, String> {
    let [id, class, verdict, origin, effects, escape, span] =
        line.split(' ').collect::<Vec<_>>()[..]
    else {
        return Err(format!("expected 7 fields in `{line}`"));
    };
    let [start, stop] = field("span", span)?.split(':').collect::<Vec<_>>()[..] else {
        return Err(format!("invalid span in `{line}`"));
    };
    Ok(Row {
        id: usize::try_from(number(field("id", id)?)?).map_err(|e| e.to_string())?,
        class: spelled(&CLASSES, "class", field("class", class)?)?,
        verdict: position(&VERDICTS, "verdict", field("verdict", verdict)?)?,
        input: Input {
            origin: position(&names(&ORIGINS), "origin", field("origin", origin)?)?,
            effects: parse_effects(field("effects", effects)?)?,
            escape: position(&names(&ESCAPES), "escape", field("escape", escape)?)?,
        },
        span: (number(start)?, number(stop)?),
    })
}

fn spelling(class: Class) -> &'static str {
    CLASSES
        .iter()
        .find(|(c, _)| *c == class)
        .map_or("", |(_, s)| *s)
}

fn print(rows: &[Row]) -> String {
    let mut out = String::from("[s3-reactivity-dump]\n\n[s3-reactivity-dump.bindings]\n");
    for row in rows {
        let effects = match row.input.effects.is_empty() {
            true => String::from("-"),
            false => (row.input.effects.iter())
                .filter_map(|e| EFFECTS.get(*e).map(|(n, _)| *n))
                .collect::<Vec<_>>()
                .join(","),
        };
        out.push_str(&format!(
            "id={} class={} verdict={} origin={} effects={effects} escape={} span={}:{}\n",
            row.id,
            spelling(row.class),
            VERDICTS.get(row.verdict).copied().unwrap_or_default(),
            ORIGINS.get(row.input.origin).map_or("", |(n, _)| *n),
            ESCAPES.get(row.input.escape).map_or("", |(n, _)| *n),
            row.span.0,
            row.span.1,
        ));
    }
    out
}

fn parse(text: &str) -> Result<Vec<Row>, String> {
    let lines: Vec<&str> = text.split('\n').filter(|l| !l.is_empty()).collect();
    let [
        "[s3-reactivity-dump]",
        "[s3-reactivity-dump.bindings]",
        rows @ ..,
    ] = lines.as_slice()
    else {
        return Err("missing reactivity dump sections".into());
    };
    let rows = rows
        .iter()
        .map(|l| parse_row(l))
        .collect::<Result<Vec<_>, _>>()?;
    if print(&rows) != text {
        return Err("fact page is not in canonical print form".into());
    }
    Ok(rows)
}

fn subset(mask: usize) -> Vec<usize> {
    (0..EFFECTS.len())
        .filter(|e| mask & (1 << e) != 0)
        .collect()
}

pub fn matrix() -> Vec<Input> {
    let mut out: Vec<Input> = (0..256)
        .map(|mask| Input {
            origin: 0,
            effects: subset(mask),
            escape: 0,
        })
        .collect();
    for origin in 0..ORIGINS.len() {
        for escape in 0..ESCAPES.len() {
            for effects in core::iter::once(Vec::new()).chain((0..EFFECTS.len()).map(|e| vec![e])) {
                out.push(Input {
                    origin,
                    effects,
                    escape,
                });
            }
        }
    }
    out
}

/// The matrix must stay exhaustive where it claims to be.
fn coverage() -> Result<(), String> {
    let mut subsets: Vec<Vec<usize>> = (0..256).map(subset).collect();
    subsets.dedup();
    subsets.sort();
    subsets.dedup();
    if subsets.len() != 256 {
        return Err("effect-set enumeration is not the full powerset".into());
    }
    let mut floors: Vec<(Class, Class, Class)> = (matrix().iter())
        .map(|i| {
            (
                floor(&ORIGINS, i.origin),
                effects_floor(&i.effects),
                floor(&ESCAPES, i.escape),
            )
        })
        .collect();
    floors.sort();
    floors.dedup();
    if floors.len() != 3 * 4 * 4 {
        return Err("matrix misses a combination of origin, effect and escape floors".into());
    }
    Ok(())
}

/// Check the page against `classifier`; returns the row count.
pub fn check(classifier: Classifier, text: &str) -> Result<usize, String> {
    coverage()?;
    let rows = parse(text)?;
    let matrix = matrix();
    if rows.len() != matrix.len() {
        return Err(format!(
            "expected {} fact rows, found {}",
            matrix.len(),
            rows.len()
        ));
    }
    for (index, (row, input)) in rows.iter().zip(&matrix).enumerate() {
        if row.id != index || row.span != (0, 0) {
            return Err(format!("row {index}: unexpected id or span"));
        }
        if row.input != *input || row.verdict != index % 3 {
            return Err(format!(
                "row {index}: input differs from the enumerated matrix"
            ));
        }
        let expected = classifier(input);
        if row.class != expected {
            return Err(format!(
                "row {index}: Rust wrote {}, the reference classifies {}",
                spelling(row.class),
                spelling(expected)
            ));
        }
    }
    Ok(rows.len())
}

/// Damaged pages and wrong classifiers must both be rejected.
pub fn negative_tests(text: &str) -> Result<(), String> {
    let rows = parse(text)?;
    let mut duplicated = rows.clone();
    duplicated.extend(rows.first().cloned());
    let damages = [
        (
            "a promoted class",
            text.replace("id=0 class=static", "id=0 class=props-stable"),
        ),
        (
            "a dropped row",
            print(rows.get(..rows.len().saturating_sub(1)).unwrap_or_default()),
        ),
        ("a duplicated row", print(&duplicated)),
        (
            "non-canonical effect order",
            text.replace("effects=freeze,capture ", "effects=capture,freeze "),
        ),
        (
            "a duplicated effect",
            text.replace("effects=freeze,capture ", "effects=freeze,freeze "),
        ),
        (
            "an unknown origin",
            text.replace("origin=prop ", "origin=props "),
        ),
        (
            "an extra field",
            text.replace("span=0:0\n", "span=0:0 extra=1\n"),
        ),
        (
            "a changed verdict",
            text.replace(
                "id=1 class=props-stable verdict=refuted",
                "id=1 class=props-stable verdict=proven",
            ),
        ),
    ];
    for (label, damaged) in damages {
        if damaged == text {
            return Err(format!("vacuous damage: {label}"));
        }
        if check(classify, &damaged).is_ok() {
            return Err(format!("lattice differential accepted {label}"));
        }
    }
    let mutants: [(&str, Classifier); 5] = [
        ("ignoring escapes", |i| {
            join(floor(&ORIGINS, i.origin), effects_floor(&i.effects))
        }),
        ("ignoring effects", |i| {
            join(floor(&ORIGINS, i.origin), floor(&ESCAPES, i.escape))
        }),
        ("ignoring origins", |i| {
            join(effects_floor(&i.effects), floor(&ESCAPES, i.escape))
        }),
        ("allocation as static", |i| {
            let effects = i.effects.iter().copied().filter(|e| *e != 7).collect();
            classify(&Input {
                effects,
                ..i.clone()
            })
        }),
        ("capture as props-stable", |i| {
            match (i.effects.as_slice(), i.origin, i.escape) {
                ([1], 0, 0) => Class::PropsStable,
                _ => classify(i),
            }
        }),
    ];
    for (label, mutant) in mutants {
        if check(mutant, text).is_ok() {
            return Err(format!(
                "lattice differential accepted a classifier {label}"
            ));
        }
    }
    Ok(())
}
