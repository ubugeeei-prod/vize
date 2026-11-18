//! A directive value the attribute printer moves onto its own line is
//! formatted for that line on the first pass. Shapes reduced from habitica.
use vize_glyph::{FormatOptions, format_sfc};

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let first = format_sfc(input, options).expect("first pass");
    assert_eq!(first.code.as_str(), expected, "first pass");
    let second = format_sfc(&first.code, options).expect("second pass");
    assert_eq!(second.code.as_str(), expected, "second pass");
    assert!(!second.changed, "second pass reports no change");
}

#[test]
fn multiline_logical_value_breaks_on_its_own_line() {
    let input = r#"<template>
  <div>
    <div>
      <div
        v-if="user.purchased.background.birthday_bash
          || user.purchased.background.on_a_strange_planet_x"
      >
        x
      </div>
      <Item
        :empty-item="!flatGear[activeItems[group]] ||
          flatGear[activeItems[group]].key.indexOf('_base_0') !== -1"
        :item="flatGear[activeItems[group]]"
      />
    </div>
  </div>
</template>
"#;
    let expected = r#"<template>
  <div>
    <div>
      <div
        v-if="
          user.purchased.background.birthday_bash ||
          user.purchased.background.on_a_strange_planet_x
        "
      >
        x
      </div>
      <Item
        :empty-item="
          !flatGear[activeItems[group]] ||
          flatGear[activeItems[group]].key.indexOf('_base_0') !== -1
        "
        :item="flatGear[activeItems[group]]"
      />
    </div>
  </div>
</template>
"#;
    assert_fixed_point(input, expected, &FormatOptions::default());
}

#[test]
fn multiline_member_value_breaks_on_its_own_line() {
    let input = r#"<template>
  <div>
    <div>
      <div
        :title="user.purchased.background.birthday_bash.someLongPropertyName
          .anotherPropertyName.yetAnotherOne"
        class="a"
      >
        x
      </div>
    </div>
  </div>
</template>
"#;
    let expected = r#"<template>
  <div>
    <div>
      <div
        :title="
          user.purchased.background.birthday_bash.someLongPropertyName.anotherPropertyName
            .yetAnotherOne
        "
        class="a"
      >
        x
      </div>
    </div>
  </div>
</template>
"#;
    assert_fixed_point(input, expected, &FormatOptions::default());
}

#[test]
fn value_budget_neighbours_keep_their_layout() {
    // A joined value that fits inline, one already standalone, one too long
    // even from column 0 (it keeps the quote on the name line), and compact
    // authored bytes that only overflow once printed.
    let input = r#"<script setup lang="ts">
const user = useUser()
</script>

<template>
  <div>
    <div>
      <div v-if="a
        || b">x</div>
      <div
        v-if="
          user.purchased.background.birthday_bash || user.purchased.background.on_a_strange
        "
      >
        x
      </div>
      <div
        v-if="user.purchased.background.birthday_bash && user.purchased.background.on_a_strange_planet
          && user.purchased.background.another_one && user.purchased.background.and_another_one_more"
      >
        x
      </div>
      <div
        v-if="userXXXXX.purchased.background.birthday_bash||user.purchased.background.on_a_strange"
        class="a"
      >
        x
      </div>
    </div>
  </div>
</template>
"#;
    let expected = r#"<script setup lang="ts">
const user = useUser();
</script>

<template>
  <div>
    <div>
      <div v-if="a || b">x</div>
      <div v-if="user.purchased.background.birthday_bash || user.purchased.background.on_a_strange">
        x
      </div>
      <div
        v-if="user.purchased.background.birthday_bash &&
          user.purchased.background.on_a_strange_planet &&
          user.purchased.background.another_one &&
          user.purchased.background.and_another_one_more"
      >
        x
      </div>
      <div
        v-if="
          userXXXXX.purchased.background.birthday_bash || user.purchased.background.on_a_strange
        "
        class="a"
      >
        x
      </div>
    </div>
  </div>
</template>
"#;
    assert_fixed_point(input, expected, &FormatOptions::default());
}

#[test]
fn value_budget_measures_tab_indentation() {
    let options = FormatOptions {
        use_tabs: true,
        ..FormatOptions::default()
    };
    let input = "<template>\n\t<div>\n\t\t<div\n\t\t\tv-if=\"user.purchased.background.birthday_bash\n\t\t\t\t|| user.purchased.background.on_a_strange_planet_xyz\"\n\t\t>\n\t\t\tx\n\t\t</div>\n\t</div>\n</template>\n";
    let expected = "<template>\n\t<div>\n\t\t<div\n\t\t\tv-if=\"\n\t\t\t\tuser.purchased.background.birthday_bash ||\n\t\t\t\tuser.purchased.background.on_a_strange_planet_xyz\n\t\t\t\"\n\t\t>\n\t\t\tx\n\t\t</div>\n\t</div>\n</template>\n";
    assert_fixed_point(input, expected, &options);
}
