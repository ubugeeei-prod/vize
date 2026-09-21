<script lang="ts">
export function shout(value: string): string {
  return value.toUpperCase();
}
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button :label="shout('primary')" />
  </variant>
  <variant name="Secondary">
    <Button :label="shout('secondary')" />
  </variant>
</art>
