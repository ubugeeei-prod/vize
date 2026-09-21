//! The template section over the real pipeline: SFC parse → Croquis
//! analysis → cross-file analyzer (S2 facts recorded per file, component
//! edges resolved through imports) → markdown.

use std::path::Path;

use vize_atelier_core::parser::parse;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileOptions};
use vize_s0::Allocator;

use super::render_complexity_markdown;

const DASHBOARD: &str = r#"<script setup lang="ts">
import DataTable from './DataTable.vue'
import StatusBadge from './StatusBadge.vue'
defineProps<{ user?: User; rows: Row[]; loading: boolean }>()
</script>

<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <StatusBadge v-if="column.key === 'status'" :active="row.active" />
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag.id">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
"#;

const DATA_TABLE: &str = r#"<script setup lang="ts">
defineProps<{ rows: Row[] }>()
</script>

<template>
  <table>
    <tr v-for="row in rows" :key="row.id">
      <td v-for="column in columns" :key="column.key">
        <slot name="cell" :row="row" :column="column">{{ row[column.key] ?? '' }}</slot>
      </td>
    </tr>
  </table>
</template>
"#;

const STATUS_BADGE: &str = r#"<script setup lang="ts">
defineProps<{ active: boolean }>()
</script>

<template>
  <span :class="active ? 'on' : 'off'">{{ active ? 'Active' : 'Inactive' }}</span>
</template>
"#;

/// Parse, analyze and register one SFC the way the CLI hosts do; a file
/// that does not parse as an SFC is skipped.
pub(super) fn add(analyzer: &mut CrossFileAnalyzer, path: &str, source: &str) {
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return;
    };
    let allocator = Allocator::new();
    let root = descriptor
        .template
        .as_ref()
        .map(|template| parse(&allocator, template.content.as_ref()).0);
    let analysis =
        analyze_sfc_descriptor_with_context(&descriptor, root.as_ref(), SfcCroquisOptions::full());
    analyzer.add_file_with_analysis(Path::new(path), source, analysis.croquis);
}

#[test]
fn the_template_section_shows_own_rendered_and_where_it_comes_from() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    add(&mut analyzer, "Dashboard.vue", DASHBOARD);
    add(&mut analyzer, "DataTable.vue", DATA_TABLE);
    add(&mut analyzer, "StatusBadge.vue", STATUS_BADGE);
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();

    let markdown = render_complexity_markdown(
        &result.complexity_report,
        &result.complexity_hotspots,
        &result.template_complexity,
    );
    let section = markdown
        .split("### Template Complexity")
        .nth(1)
        .and_then(|rest| rest.split("### Top Hotspots").next())
        .expect("the template section is rendered");
    #[allow(clippy::disallowed_macros)]
    {
        insta::assert_snapshot!(section);
    }
}
