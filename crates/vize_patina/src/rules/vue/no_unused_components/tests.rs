use super::NoUnusedComponents;
use crate::linter::Linter;
use crate::rule::{Rule, RuleCategory};

#[test]
fn test_meta() {
    let rule = NoUnusedComponents::default();
    assert_eq!(rule.meta().name, "vue/no-unused-components");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
}

#[test]
fn test_should_ignore() {
    let rule = NoUnusedComponents::default();
    assert!(rule.should_ignore("_Internal"));
    assert!(!rule.should_ignore("MyComponent"));
}

fn lint_messages(sfc: &str) -> Vec<String> {
    Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-components".into()]))
        .lint_sfc(sfc, "test.vue")
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect()
}

#[test]
fn test_plain_script_component_value_is_not_registration() {
    let sfc = r#"<template>
  <DocSections :docs="docs" />
</template>

<script>
import DocApiTable from './DocApiTable.vue'

export default {
  data() {
    return {
      docs: [{ component: DocApiTable }]
    }
  }
}
</script>
"#;

    assert_eq!(lint_messages(sfc), Vec::<String>::new());
}

#[test]
fn test_options_api_registration_reports_public_alias() {
    let sfc = r#"<script>
import Style from './style.vue'

export default {
  components: {
    FourStyle: Style,
  },
}

</script>

<template>
  <div />
</template>
"#;

    assert_eq!(
        lint_messages(sfc),
        vec!["Component 'FourStyle' is registered but never used in template"]
    );
}

#[test]
fn unused_script_setup_import_points_to_import_name() {
    let sfc = "<script setup>\nimport Child from './Child.vue'\n</script>\n<template><div /></template>\n";
    let diagnostics = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-components".into()]))
        .lint_sfc(sfc, "test.vue")
        .diagnostics;
    assert_eq!(diagnostics.len(), 1);
    let start = sfc.find("Child from").unwrap();
    assert_eq!(
        (diagnostics[0].start, diagnostics[0].end),
        (start as u32, (start + 5) as u32)
    );
}

#[test]
fn unused_options_registration_points_to_public_name() {
    let sfc = "<script>\nimport Style from './style.vue'\nexport default { components: { FourStyle: Style } }\n</script>\n<template><div /></template>\n";
    let diagnostics = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-components".into()]))
        .lint_sfc(sfc, "test.vue")
        .diagnostics;
    assert_eq!(diagnostics.len(), 1);
    let start = sfc.find("FourStyle:").unwrap();
    assert_eq!(
        (diagnostics[0].start, diagnostics[0].end),
        (start as u32, (start + 9) as u32)
    );
}

#[test]
fn test_registered_library_component_reports_unused() {
    let sfc = r#"<script>
import { SfButton } from '@storefront-ui/vue'

export default {
  components: {
    SfButton,
  },
}
</script>

<template>
  <div />
</template>
"#;

    assert_eq!(
        lint_messages(sfc),
        vec!["Component 'SfButton' is registered but never used in template"]
    );
}

#[test]
fn issue_6690_typeof_component_import_is_used_in_script_setup() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue';
import Child from './Child.vue';

const childRef = ref<InstanceType<typeof Child>>();
defineExpose({ childRef });
</script>

<template><div>hello</div></template>
"#;

    assert_eq!(lint_messages(sfc), Vec::<String>::new());
}

#[test]
fn component_name_in_type_string_is_not_a_reference() {
    let sfc = r#"<script setup lang="ts">
import Child from './Child.vue';
type Label = 'Child';
</script>

<template><div>hello</div></template>
"#;

    assert_eq!(
        lint_messages(sfc),
        vec!["Component 'Child' is registered but never used in template"]
    );
}

#[test]
fn component_name_prefixes_and_slot_names_do_not_mark_registrations_used() {
    let sfc = r#"<template>
  <MdTableRowGhost v-if="$scopedSlots['md-table-row']" />
  <slot name="md-table-row" />
</template>
<script>
import MdTableRow from './MdTableRow'
import MdTableRowGhost from './MdTableRowGhost'
import MdTableCellSelection from './MdTableCellSelection'
export default {
  components: { MdTableRow, MdTableRowGhost, MdTableCellSelection }
}
</script>"#;
    assert_eq!(
        lint_messages(sfc),
        vec![
            "Component 'MdTableCellSelection' is registered but never used in template",
            "Component 'MdTableRow' is registered but never used in template",
        ]
    );
}

#[test]
fn foster_parenting_does_not_hide_provably_unused_registrations() {
    let sfc = r#"<template>
  <table>
    <tbody>
      <MdTableRowGhost />
    </tbody>
  </table>
</template>
<script>
import MdTableRow from './MdTableRow'
import MdTableRowGhost from './MdTableRowGhost'
import MdTableCellSelection from './MdTableCellSelection'
export default {
  components: { MdTableRow, MdTableRowGhost, MdTableCellSelection }
}
</script>"#;
    assert_eq!(
        lint_messages(sfc),
        vec![
            "Component 'MdTableCellSelection' is registered but never used in template",
            "Component 'MdTableRow' is registered but never used in template",
        ]
    );
}

#[test]
fn complex_fostered_table_keeps_unused_registration_evidence() {
    let sfc = r#"<template>
  <md-tag-switcher :md-tag="contentTag" class="md-table">
    <slot name="md-table-toolbar" />
    <keep-alive>
      <md-table-alternate-header v-if="$scopedSlots['md-table-alternate-header'] && selectedCount">
        <slot name="md-table-alternate-header" :count="selectedCount" />
      </md-table-alternate-header>
    </keep-alive>
    <div v-if="mdFixedHeader"><table><md-table-thead /></table></div>
    <md-content>
      <table>
        <md-table-thead v-if="$scopedSlots['md-table-row']" />
        <tbody v-if="!$scopedSlots['md-table-row']"><slot /></tbody>
        <tbody v-else-if="value.length">
          <md-table-row-ghost v-for="item in value" :key="item.id">
            <slot name="md-table-row" :item="item" />
          </md-table-row-ghost>
        </tbody>
        <tbody v-else-if="$scopedSlots['md-table-empty-state']"><tr><td><slot /></td></tr></tbody>
      </table>
    </md-content>
  </md-tag-switcher>
</template>
<script>
import MdTagSwitcher from './MdTagSwitcher'
import MdTableThead from './MdTableThead'
import MdTableAlternateHeader from './MdTableAlternateHeader'
import MdTableRow from './MdTableRow'
import MdTableRowGhost from './MdTableRowGhost'
import MdTableCellSelection from './MdTableCellSelection'
export default {
  components: { MdTagSwitcher, MdTableAlternateHeader, MdTableThead, MdTableRow, MdTableRowGhost, MdTableCellSelection }
}
</script>"#;
    assert_eq!(
        lint_messages(sfc),
        vec![
            "Foster parenting moved this element before the nearest open table.",
            "Component 'MdTableRow' is registered but never used in template",
            "Component 'MdTableCellSelection' is registered but never used in template",
        ]
    );
}
