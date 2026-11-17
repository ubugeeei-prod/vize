use super::mentions_pug_language;

#[test]
fn the_pug_gate_matches_either_spelling_in_any_ascii_case() {
    for (source, mentioned) in [
        (r#"<template lang="pug">div</template>"#, true),
        (r#"<template lang="PUG">div</template>"#, true),
        (r#"<template lang='Jade'>div</template>"#, true),
        ("pug", true),
        ("xJADE", true),
        ("a pUg b", true),
        ("<template lang=\"html\">puppy jaded</template>", true),
        ("", false),
        ("pu", false),
        ("jad", false),
        ("<template><p>plug-in</p></template>", false),
        ("p u g j a d e", false),
    ] {
        assert_eq!(mentions_pug_language(source), mentioned, "{source}");
    }
}
