<script setup lang="ts">
function inspect(count: number, title: string, flag: boolean) {
  if (count) {}
  while (title) { break; }
  do {} while (count);
  for (; title;) { break; }
  const result = count ? 1 : 2;
  const negated = !title;
  const value = count && title;
  const allowedValue = flag && title;
  if (count && title) {}
  if (count || title) {}
  if (count ?? 0) {}
  if (count === 0 || title.length > 0) {}
  const emoji = "😀"; if (title) {}
  return [result, negated, value, allowedValue, emoji];
}
</script>
