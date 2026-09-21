<script setup lang="ts">
const simpleLabel: string = "型付き"
const props = defineProps<{ title: string }>()
</script>

<art title="ボタン" component="./Button.vue">
  <variant name="Primary" default>
    <p>ラベル → {{ simpleLabel }} ・ 見出し ✅ {{ props.title }}</p>
  </variant>
</art>
