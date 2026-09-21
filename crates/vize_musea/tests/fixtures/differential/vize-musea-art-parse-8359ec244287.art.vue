<art title="Button" description="A versatile button component" component="./Button.vue" category="atoms" tags="ui,input,interactive">
  <variant name="Primary" default>
    <Button variant="primary">Primary Button</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>
  <variant name="Outline">
    <Button variant="outline">Outline Button</Button>
  </variant>
  <variant name="Ghost">
    <Button variant="ghost">Ghost Button</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Disabled Button</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>

<style scoped>
.art-container {
  padding: 20px;
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}
</style>
