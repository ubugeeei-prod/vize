<script setup lang="ts">
import { inject, provide, ref } from "vue";
import type { InjectionKey, Ref } from "vue";
const ItemsKey: InjectionKey<Ref<string[]>> = Symbol("items");
const items = ref<string[]>([]);
provide("items", items);
provide(ItemsKey, items);
const fromString = inject<Ref<string[]>>("items");
const fromKey = inject(ItemsKey);
void fromString;
void fromKey;
</script>
