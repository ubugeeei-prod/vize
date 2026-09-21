<script lang="ts">
export function regularFormat(value: Date, locale: string): string {
  return value.toLocaleDateString(locale)
}
</script>
<art title="Regular">
  <variant name="Regular">
    <p>{{ regularFormat(new Date(), 'en') }}</p>
  </variant>
</art>
