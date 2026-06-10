//! Snapshot tests for vize_canon.

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod virtual_ts_tests {
    use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

    /// Generate virtual TypeScript from SFC using canon's type_check_sfc.
    /// This uses croquis scope analysis to generate proper JavaScript scoping
    /// (for-of loops, closures, IIFEs) instead of declare statements.
    fn generate_virtual_ts_from_sfc(source: &str) -> vize_carton::String {
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        result.virtual_ts.unwrap_or_default()
    }

    #[test]
    fn snapshot_virtual_ts_simple_component() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
const message = ref('Hello')

function increment() {
  count.value++
}
</script>

<template>
  <div>
    <p>{{ message }}</p>
    <p>Count: {{ count }}</p>
    <button @click="increment">+1</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_simple_component", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_props() {
        let source = r#"<script setup lang="ts">
interface Props {
  title: string
  count?: number
}

const props = defineProps<Props>()
</script>

<template>
  <h1>{{ props.title }}</h1>
  <p v-if="props.count">Count: {{ props.count }}</p>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_props", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_emits() {
        let source = r#"<script setup lang="ts">
interface Emits {
  (e: 'update', value: number): void
  (e: 'close'): void
}

const emit = defineEmits<Emits>()

function handleClick() {
  emit('update', 42)
}
</script>

<template>
  <button @click="handleClick">Update</button>
  <button @click="emit('close')">Close</button>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_emits", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_v_for() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const items = ref([1, 2, 3])
</script>

<template>
  <ul>
    <li v-for="(item, index) in items" :key="index">
      {{ index }}: {{ item }}
    </li>
  </ul>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_v_for", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_slots() {
        let source = r#"<script setup lang="ts">
import { useSlots } from 'vue'

const slots = useSlots()
</script>

<template>
  <div>
    <slot name="header" :title="'Header'"></slot>
    <slot></slot>
    <slot name="footer"></slot>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_slots", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_complex_component() {
        let source = r#"<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface Props {
  initialCount?: number
  title: string
}

interface Emits {
  (e: 'change', value: number): void
}

const props = withDefaults(defineProps<Props>(), {
  initialCount: 0
})

const emit = defineEmits<Emits>()

const count = ref(props.initialCount)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value++
  emit('change', count.value)
}

watch(count, (newVal) => {
  console.log('Count changed:', newVal)
})
</script>

<template>
  <div class="counter">
    <h1>{{ props.title }}</h1>
    <p>Count: {{ count }}</p>
    <p>Doubled: {{ doubled }}</p>
    <button @click="increment">+1</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_complex_component", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_composables() {
        let source = r#"<script setup lang="ts">
import { useMouse } from '@vueuse/core'

const { x, y } = useMouse()
</script>

<template>
  <div>
    Mouse position: {{ x }}, {{ y }}
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_composables", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_v_for_destructuring() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

interface Item {
  id: number
  name: string
}

const items = ref<Item[]>([])
</script>

<template>
  <ul>
    <li v-for="{ id, name } in items" :key="id">
      {{ id }}: {{ name }}
    </li>
  </ul>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_v_for_destructuring", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_nested_v_if_v_else() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const status = ref<'loading' | 'error' | 'success'>('loading')
const message = ref('')
const data = ref<string[]>([])
</script>

<template>
  <div>
    <div v-if="status === 'loading'">Loading...</div>
    <div v-else-if="status === 'error'">
      Error: {{ message }}
    </div>
    <div v-else>
      <p v-for="item in data" :key="item">{{ item }}</p>
    </div>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_nested_v_if_v_else", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_scoped_slots() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
import MyList from './MyList.vue'

const items = ref(['a', 'b', 'c'])
</script>

<template>
  <MyList :items="items">
    <template #default="{ item, index }">
      <span>{{ index }}: {{ item }}</span>
    </template>
    <template #header="{ title }">
      <h1>{{ title }}</h1>
    </template>
  </MyList>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        assert!(
            virtual_ts.contains(
                r#"void function _slot_default({ item, index }: typeof MyList extends { new (): { $slots: infer __S } } ? (__S extends { "default"?: (props: infer __P"#
            ),
            "<template #default> slot props should be typed from the owning component:\n{virtual_ts}"
        );
        assert!(
            !virtual_ts.contains(r#"void function _slot_default({ item, index }: any)"#),
            "<template #default> slot props must not fall back to any:\n{virtual_ts}"
        );
        insta::assert_snapshot!("virtual_ts_scoped_slots", virtual_ts);
    }

    #[test]
    fn virtual_ts_dynamic_component_v_slot_uses_slot_prop_union() {
        let source = r#"<script setup lang="ts">
import MyList from './MyList.vue'

const slot = 'items'
const items = ['a', 'b']
</script>

<template>
  <MyList :items="items" v-slot:[slot]="{ item }">{{ item }}</MyList>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);

        assert!(
            virtual_ts.contains("[__K in keyof __S]: NonNullable<__S[__K]>"),
            "dynamic slot names should infer from all declared slot props:\n{virtual_ts}"
        );
        assert!(
            !virtual_ts.contains(r#"__S extends { "slot"?: (props: infer __P"#),
            "dynamic slot expression must not be injected as a static slot key:\n{virtual_ts}"
        );
    }

    #[test]
    fn snapshot_virtual_ts_v_model() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const text = ref('')
const checked = ref(false)
const selected = ref('option1')
</script>

<template>
  <div>
    <input v-model="text" />
    <input type="checkbox" v-model="checked" />
    <select v-model="selected">
      <option value="option1">Option 1</option>
      <option value="option2">Option 2</option>
    </select>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_v_model", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_define_model() {
        let source = r#"<script setup lang="ts">
const model = defineModel<string>({ required: true })
const count = defineModel<number>('count', { default: 0 })
</script>

<template>
  <input v-model="model" />
  <button @click="$emit('update:count', count + 1)">{{ count }}</button>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_define_model", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_template_refs() {
        let source = r#"<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'

const inputRef = ref<HTMLInputElement | null>(null)
const buttonEl = useTemplateRef('btn')
</script>

<template>
  <div>
    <input ref="inputRef" />
    <button ref="btn">Click</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_template_refs", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_generic_component() {
        let source = r#"<script setup lang="ts" generic="T extends string | number">
import { ref } from 'vue'

const props = defineProps<{
  items: T[]
  selected?: T
}>()

const emit = defineEmits<{
  (e: 'select', item: T): void
}>()

const activeItem = ref<T | null>(null)
</script>

<template>
  <div>
    <div v-for="item in props.items" :key="String(item)">
      {{ item }}
    </div>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_generic_component", virtual_ts);
    }

    #[test]
    fn virtual_ts_annotated_slot_props_keep_user_annotation_and_check_it() {
        // `#item="{ element }: { element: Tag }"` (a typed slot scope, used by
        // e.g. voicevox) must not get the inferred slot type appended after
        // the user's own annotation — `pattern: A: B` is a syntax error that
        // aborts Corsa's semantic pass for the whole project. The user
        // annotation types the bindings, and a separate assignment asserts the
        // child's actual slot props are assignable to it.
        let source = r#"<script setup lang="ts">
import Draggable from './Draggable.vue'
type Tag = { id: string }
</script>

<template>
  <Draggable>
    <template #item="{ element }: { element: Tag }">
      <div>{{ element.id }}</div>
    </template>
  </Draggable>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        assert!(
            virtual_ts.contains("({ element }: { element: Tag })"),
            "slot bindings must use the user's own annotation:\n{virtual_ts}"
        );
        assert!(
            virtual_ts.contains("const __slot_annotation_check: { element: Tag } ="),
            "the child's actual slot props must be checked against the annotation:\n{virtual_ts}"
        );
        assert!(
            !virtual_ts.contains("{ element: Tag }: typeof"),
            "the inferred slot type must not be appended after the annotation:\n{virtual_ts}"
        );
    }

    #[test]
    fn virtual_ts_const_generic_component_strips_const_from_type_positions() {
        // `generic="const T extends ..."` (TS 5.0 const type parameter, used by
        // e.g. misskey's MkTabs) — the parameter NAME is `T`, and the `const`
        // modifier is only legal on function/method type parameters, never on
        // the generated `type Props<...>` alias (TS1277).
        let source = r#"<script setup lang="ts" generic="const T extends string | number">
const props = defineProps<{
  items: T[]
}>()
</script>

<template>
  <div>{{ props.items }}</div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        assert!(
            virtual_ts.contains("export type Props<T extends string | number = any>"),
            "Props alias must declare `T` without the const modifier:\n{virtual_ts}"
        );
        assert!(
            virtual_ts.contains("function __setup<const T extends string | number>()"),
            "the setup closure keeps the const modifier (legal on functions):\n{virtual_ts}"
        );
        assert!(
            !virtual_ts.contains("Props<const>"),
            "the const modifier must never be treated as the parameter name:\n{virtual_ts}"
        );
    }

    #[test]
    fn snapshot_virtual_ts_dynamic_component() {
        let source = r#"<script setup lang="ts">
import { ref, markRaw } from 'vue'
import CompA from './CompA.vue'
import CompB from './CompB.vue'

const currentComponent = ref(markRaw(CompA))

function switchComponent() {
  currentComponent.value = currentComponent.value === CompA ? markRaw(CompB) : markRaw(CompA)
}
</script>

<template>
  <div>
    <component :is="currentComponent"></component>
    <button @click="switchComponent">Switch</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_dynamic_component", virtual_ts);
    }
}
