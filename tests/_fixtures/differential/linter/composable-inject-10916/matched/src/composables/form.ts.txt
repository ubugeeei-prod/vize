import { inject, onBeforeUnmount, onMounted } from "vue";
import type { InjectionKey } from "vue";

export interface FormApi {
  register(field: { validate(): boolean }): void;
  unregister(field: { validate(): boolean }): void;
}
export const FormKey: InjectionKey<FormApi> = Symbol("form");

export function useFormField(validate: () => boolean): void {
  const form = inject(FormKey, null);
  const field = { validate };
  onMounted(() => form?.register(field));
  onBeforeUnmount(() => form?.unregister(field));
}

export function useUncalledField(): void {
  inject(Symbol("uncalled"));
}
