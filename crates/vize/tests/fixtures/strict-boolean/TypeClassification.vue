<script setup lang="ts">
type Alias = { name: string } | { id: number };
type Label = string;
enum State { Off, On }
function inspect(object: Alias | undefined, label: Label | null, amount: number | undefined, enabled: boolean | null, unknownValue: unknown, anyValue: any, mixed: string | number, state: State | undefined, impossible: never, nullValue: null, undefinedValue: undefined) {
  if (object) {}
  if (label) {}
  if (amount) {}
  if (enabled) { if (enabled) {} }
  if (unknownValue) {}
  if (anyValue) {}
  if (mixed) {}
  if (state) {}
  if (impossible) {}
  if (nullValue) {}
  if (undefinedValue) {}
  if (label !== null) { if (label) {} }
  if (object !== undefined) { if (object) {} }
}
</script>
