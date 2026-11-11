use super::parse_script_setup;

fn assert_only_pet(source: &str) {
    let result = parse_script_setup(source);
    let names: Vec<_> = result
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str())
        .collect();
    assert_eq!(names, ["pet"]);
}

#[test]
fn commented_union_member_is_not_a_prop() {
    assert_only_pet(
        r#"
const { pet } = defineProps<{
  pet:
    | // cats
      Cat
    // dogs
    | Dog
    | Bird;
}>();
"#,
    );
}

#[test]
fn one_union_comment_still_names_only_the_prop() {
    assert_only_pet(
        r#"
const { pet } = defineProps<{
  pet:
    | // cats
      Cat
    | Dog
    | Bird;
}>();
"#,
    );
    assert_only_pet(
        r#"
const { pet } = defineProps<{
  pet:
    | Cat
    // dogs
    | Dog
    | Bird;
}>();
"#,
    );
}
