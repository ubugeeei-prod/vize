<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('primary', 2)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 3)" />
  </variant>
</art>
