<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

const broken: number = "not a number";

function format(value: string, precision: number): string {
  return value.slice(0, precision);
}
</script>

<art>
  <variant name="Primary" default>
    <Button :label="format('primary', broken)" />
  </variant>
  <variant name="Secondary">
    <Button :label="format('secondary', 3)" />
  </variant>
  <variant name="Tertiary">
    <Button :label="format('tertiary', 4)" />
  </variant>
</art>
