<script setup lang="ts">
import { ref } from "vue";
const list = ref<HTMLElement>();
const SELECTED = "[aria-selected=true]";
function focusSelected() {
  const el = list.value?.querySelector<HTMLElement>(SELECTED);
  if (!el) return;
  el.focus();
}
</script>
