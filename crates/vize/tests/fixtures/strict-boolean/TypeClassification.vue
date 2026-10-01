<script setup lang="ts">
type Alias = { name: string } | { id: number };
type Label = string;
enum State { Off, On }
function inspect(object: Alias | undefined, label: Label | null, amount: number | undefined, enabled: boolean | null, unknownValue: unknown, anyValue: any, mixed: string | number, state: State | undefined, impossible: never) {
  if (object) {}
  if (label) {}
  if (amount) {}
  if (enabled) { if (enabled) {} }
  if (unknownValue) {}
  if (anyValue) {}
  if (mixed) {}
  if (state) {}
  if (impossible) {}
  if (null) {}
  if (undefined) {}
  if (label !== null) { if (label) {} }
  if (object !== undefined) { if (object) {} }
}
</script>
