//! Whole-application regressions on small reported projects.
use super::{analysis::analyze_application, discovery::discover_sources};
use std::{fs, path::Path};
use vize_l0::{String, cstr};

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn doctor_codes(root: &Path) -> Vec<String> {
    let sources = discover_sources(root, &[".".into()]).unwrap();
    let report = analyze_application(&root.canonicalize().unwrap(), &sources, false).unwrap();
    report
        .findings()
        .iter()
        .map(|finding| finding.code.as_str().into())
        .collect()
}

/// #10776: Nuxt `srcDir` / `dir.pages` / `dir.layouts` keep layout and page ancestry.
#[test]
fn nuxt_custom_dirs_keep_layout_provides_matched() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(
        root,
        "nuxt.config.ts",
        r#"export default defineNuxtConfig({
  ssr: false,
  srcDir: "src",
  dir: { layouts: "nuxt/layouts", pages: "nuxt/pages", app: "nuxt/app" }
});"#,
    );
    write(
        root,
        "src/features/keys.ts",
        r#"import type { InjectionKey } from "vue";
export const NoticeKey: InjectionKey<{ show(text: string): void }> = Symbol("notice");"#,
    );
    write(
        root,
        "src/nuxt/app/app.vue",
        "<template>\n  <NuxtLayout>\n    <NuxtPage />\n  </NuxtLayout>\n</template>\n",
    );
    write(
        root,
        "src/nuxt/layouts/default.vue",
        r#"<script setup lang="ts">
import { provide } from "vue";
import { NoticeKey } from "../../features/keys";

provide(NoticeKey, { show: text => console.warn(text) });
</script>

<template>
  <main><slot /></main>
</template>
"#,
    );
    write(
        root,
        "src/features/NoticeButton.vue",
        r#"<script setup lang="ts">
import { inject } from "vue";
import { NoticeKey } from "./keys";

const notice = inject(NoticeKey)!;
</script>

<template>
  <button type="button" @click="notice.show('hi')">notify</button>
</template>
"#,
    );
    write(
        root,
        "src/nuxt/pages/index.vue",
        r#"<script setup lang="ts">
import NoticeButton from "../../features/NoticeButton.vue";
</script>

<template>
  <NoticeButton />
</template>
"#,
    );

    assert_eq!(doctor_codes(root), Vec::<String>::new());
}

/// #10608: `defineModel` props, imported alias prop types and fallthrough attrs are legal usage.
#[test]
fn model_alias_and_fallthrough_props_are_not_findings() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(
        root,
        "tsconfig.json",
        r##"{ "compilerOptions": { "paths": { "#shared/*": ["./src/shared/*"] } } }"##,
    );
    write(
        root,
        "src/shared/item-property.ts",
        "export type ItemProperty = string | ((item: Record<string, unknown>) => string);\n",
    );
    write(
        root,
        "src/child-model.vue",
        r#"<script setup lang="ts">
const keyword = defineModel<string>("keyword", { required: true });
const { label } = defineProps<{ label: string }>();
</script>

<template>
  <label>{{ label }}<input v-model="keyword" /></label>
</template>
"#,
    );
    write(
        root,
        "src/child-alias.vue",
        r##"<script setup lang="ts">
import type { ItemProperty } from "#shared/item-property";

const { itemText = "text" } = defineProps<{ itemText?: ItemProperty }>();
</script>

<template>
  <span>{{ typeof itemText }}</span>
</template>
"##,
    );
    write(
        root,
        "src/parent.vue",
        r#"<script setup lang="ts">
import { ref } from "vue";
import ChildAlias from "./child-alias.vue";
import ChildModel from "./child-model.vue";

const keyword = ref("");
</script>

<template>
  <div>
    <ChildModel v-model:keyword="keyword" label="Search" aria-label="search box" />
    <ChildAlias item-text="name" />
  </div>
</template>
"#,
    );

    assert_eq!(doctor_codes(root), Vec::<String>::new());
}

/// #10608: an `$env` block that cannot enable SSR keeps a client-only app client-only.
#[test]
fn client_only_nuxt_env_blocks_keep_ssr_findings_off() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write(
        root,
        "src/features/Mover.vue",
        r#"<script setup lang="ts">
import { ref, watch } from "vue";

const { open = false } = defineProps<{ open?: boolean }>();
const root = ref<HTMLElement | null>(null);
function moveToBody(): void {
  const host = document.querySelector("[data-app]") ?? document.body;
  if (root.value && root.value.parentElement !== host) host.appendChild(root.value);
}
watch(
  () => open,
  opened => {
    if (opened) moveToBody();
  }
);
</script>

<template>
  <div ref="root"><slot /></div>
</template>
"#,
    );
    write(
        root,
        "src/pages/index.vue",
        r#"<script setup lang="ts">
import Mover from "../features/Mover.vue";
</script>

<template>
  <Mover :open="true">content</Mover>
</template>
"#,
    );
    write(
        root,
        "src/app.vue",
        "<template>\n  <NuxtPage />\n</template>\n",
    );
    for (override_block, server_rendered) in [
        ("$development: {}", false),
        ("$development: { devtools: { enabled: false } }", false),
        ("$production: { ssr: false }", false),
        ("$production: { ssr: true }", true),
    ] {
        write(
            root,
            "nuxt.config.ts",
            &cstr!(
                r#"export default defineNuxtConfig({{ ssr: false, srcDir: "src", {override_block} }});"#
            ),
        );
        let ssr = String::from("VIZE_DOCTOR_CF_BROWSER_API_SSR");
        assert_eq!(
            doctor_codes(root),
            vec![ssr; if server_rendered { 2 } else { 0 }],
            "{override_block}"
        );
    }
}
