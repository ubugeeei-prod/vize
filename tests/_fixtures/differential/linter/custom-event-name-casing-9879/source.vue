<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string];
  "clear-click": [];
}>();
emit("update:modelValue", "ready");
emit("clear-click");
</script>
