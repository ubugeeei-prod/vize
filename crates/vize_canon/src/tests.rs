//! Snapshot tests for vize_canon.

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod virtual_ts_tests {
    use crate::sfc_typecheck::{type_check_sfc, SfcTypeCheckOptions};

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
        insta::assert_snapshot!("virtual_ts_scoped_slots", virtual_ts);
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
    <component :is="currentComponent" />
    <button @click="switchComponent">Switch</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_dynamic_component", virtual_ts);
    }
}
