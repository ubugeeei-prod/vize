<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, useId } from "vue";
import type { DesignToken } from "../../api";

const props = defineProps<{
  isOpen: boolean;
  mode: "create" | "edit";
  editPath?: string;
  editToken?: DesignToken;
  primitiveTokenPaths: string[];
  existingPaths: string[];
}>();

const emit = defineEmits<{
  close: [];
  submit: [path: string, token: Omit<DesignToken, "$resolvedValue">];
}>();

const NO_TOKEN_TYPE = "__none__";
const tokenPath = ref("");
const tokenValue = ref<string>("");
const tokenType = ref(NO_TOKEN_TYPE);
const tokenDescription = ref("");
const tier = ref<"primitive" | "semantic">("primitive");
const reference = ref("");
const validationError = ref<string | null>(null);
const modalRef = ref<HTMLElement | null>(null);
const pathInputRef = ref<HTMLInputElement | null>(null);
const fieldIds = {
  title: useId(),
  path: useId(),
  tier: useId(),
  value: useId(),
  reference: useId(),
  type: useId(),
  description: useId(),
};

const TOKEN_TYPES = [
  "color",
  "dimension",
  "spacing",
  "fontSize",
  "fontWeight",
  "lineHeight",
  "shadow",
  "borderRadius",
  "opacity",
  "string",
  "number",
];

watch(
  () => props.isOpen,
  (open) => {
    if (open) {
      if (props.mode === "edit" && props.editToken && props.editPath) {
        tokenPath.value = props.editPath;
        tokenValue.value = String(props.editToken.value);
        tokenType.value = props.editToken.type || NO_TOKEN_TYPE;
        tokenDescription.value = props.editToken.description ?? "";
        tier.value = props.editToken.$tier ?? "primitive";
        reference.value = props.editToken.$reference ?? "";
      } else {
        tokenPath.value = "";
        tokenValue.value = "";
        tokenType.value = NO_TOKEN_TYPE;
        tokenDescription.value = "";
        tier.value = "primitive";
        reference.value = "";
      }
      validationError.value = null;
      nextTick(() => {
        if (pathInputRef.value?.disabled) {
          modalRef.value?.querySelector<HTMLInputElement>("input:not([disabled])")?.focus();
        } else {
          pathInputRef.value?.focus();
        }
      });
    }
  },
);

const referenceOptions = computed(() => {
  if (!reference.value) return props.primitiveTokenPaths;
  const q = reference.value.toLowerCase();
  return props.primitiveTokenPaths.filter((p) => p.toLowerCase().includes(q));
});

const title = computed(() => (props.mode === "create" ? "Add Token" : "Edit Token"));

function validate(): boolean {
  if (!tokenPath.value.trim()) {
    validationError.value = "Token path is required";
    return false;
  }
  if (props.mode === "create" && props.existingPaths.includes(tokenPath.value)) {
    validationError.value = "A token already exists at this path";
    return false;
  }
  if (tier.value === "semantic" && !reference.value.trim()) {
    validationError.value = "Semantic tokens require a reference";
    return false;
  }
  if (tier.value === "primitive" && !tokenValue.value.trim()) {
    validationError.value = "Token value is required";
    return false;
  }
  validationError.value = null;
  return true;
}

function handleSubmit() {
  if (!validate()) return;

  const token: Omit<DesignToken, "$resolvedValue"> = {
    value: tier.value === "semantic" ? `{${reference.value}}` : tokenValue.value,
    $tier: tier.value,
  };
  if (tokenType.value !== NO_TOKEN_TYPE) token.type = tokenType.value;
  if (tokenDescription.value) token.description = tokenDescription.value;
  if (tier.value === "semantic") token.$reference = reference.value;

  emit("submit", tokenPath.value, token);
}

function selectReference(path: string) {
  reference.value = path;
}

function closeForm() {
  emit("close");
}

function onModalKeydown(event: KeyboardEvent) {
  if (!props.isOpen) return;
  if (event.key === "Escape") {
    closeForm();
    return;
  }
  if (event.key !== "Tab") return;

  const focusable = modalRef.value?.querySelectorAll<HTMLElement>(
    "button:not([disabled]), input:not([disabled]), select:not([disabled])",
  );
  if (!focusable?.length) return;
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  if (!(event.target instanceof Node) || !modalRef.value?.contains(event.target)) {
    event.preventDefault();
    first.focus();
  } else if (event.shiftKey && event.target === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && event.target === last) {
    event.preventDefault();
    first.focus();
  }
}

onMounted(() => document.addEventListener("keydown", onModalKeydown));
onUnmounted(() => document.removeEventListener("keydown", onModalKeydown));
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="isOpen" class="modal-overlay">
        <button
          type="button"
          class="modal-backdrop"
          aria-label="Close token form"
          tabindex="-1"
          @click="closeForm"
        />
        <div
          ref="modalRef"
          class="modal-content"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="fieldIds.title"
        >
          <div class="modal-header">
            <h2 :id="fieldIds.title" class="modal-title">{{ title }}</h2>
            <button
              type="button"
              class="modal-close"
              aria-label="Close token form"
              @click="closeForm"
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          <form class="modal-form" @submit.prevent="handleSubmit">
            <div class="form-field">
              <label class="form-label" :for="fieldIds.path">Token Path</label>
              <input
                :id="fieldIds.path"
                ref="pathInputRef"
                v-model="tokenPath"
                class="form-input token-form-path-input"
                :disabled="mode === 'edit'"
                placeholder="e.g. color.primary.500"
              />
            </div>

            <div class="form-field">
              <span :id="fieldIds.tier" class="form-label">Tier</span>
              <div class="tier-radio-group" role="radiogroup" :aria-labelledby="fieldIds.tier">
                <label class="tier-radio" :class="{ 'tier-radio--active': tier === 'primitive' }">
                  <input
                    v-model="tier"
                    type="radio"
                    :name="fieldIds.tier"
                    value="primitive"
                    class="tier-radio-input"
                  />
                  <span class="tier-radio-label">Primitive</span>
                </label>
                <label class="tier-radio" :class="{ 'tier-radio--active': tier === 'semantic' }">
                  <input
                    v-model="tier"
                    type="radio"
                    :name="fieldIds.tier"
                    value="semantic"
                    class="tier-radio-input"
                  />
                  <span class="tier-radio-label">Semantic</span>
                </label>
              </div>
            </div>

            <template v-if="tier === 'primitive'">
              <div class="form-field">
                <label class="form-label" :for="fieldIds.value">Value</label>
                <input
                  :id="fieldIds.value"
                  v-model="tokenValue"
                  class="form-input"
                  placeholder="e.g. #3b82f6, 16px, 400"
                />
              </div>
            </template>

            <template v-else>
              <div class="form-field">
                <label class="form-label" :for="fieldIds.reference">Reference</label>
                <input
                  :id="fieldIds.reference"
                  v-model="reference"
                  class="form-input"
                  placeholder="e.g. color.blue.500"
                />
                <div v-if="referenceOptions.length > 0" class="reference-list">
                  <button
                    v-for="opt in referenceOptions.slice(0, 8)"
                    :key="opt"
                    type="button"
                    class="reference-option"
                    :class="{ 'reference-option--selected': opt === reference }"
                    @click="() => selectReference(opt)"
                  >
                    {{ opt }}
                  </button>
                </div>
              </div>
            </template>

            <div class="form-field">
              <label class="form-label" :for="fieldIds.type">Type</label>
              <select :id="fieldIds.type" v-model="tokenType" class="form-input form-select">
                <option :value="NO_TOKEN_TYPE">None</option>
                <option v-for="t in TOKEN_TYPES" :key="t" :value="t">{{ t }}</option>
              </select>
            </div>

            <div class="form-field">
              <label class="form-label" :for="fieldIds.description">Description</label>
              <input
                :id="fieldIds.description"
                v-model="tokenDescription"
                class="form-input"
                placeholder="Optional description"
              />
            </div>

            <div v-if="validationError" class="form-error">
              {{ validationError }}
            </div>

            <div class="modal-footer">
              <button type="button" class="btn btn--secondary" @click="closeForm">Cancel</button>
              <button type="submit" class="btn btn--primary">
                {{ mode === "create" ? "Create" : "Save" }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--musea-layer-modal);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--musea-overlay);
  backdrop-filter: blur(4px);
}

.modal-backdrop {
  position: absolute;
  inset: 0;
  border: 0;
  background: transparent;
  cursor: default;
}

.modal-content {
  position: relative;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-lg, 12px);
  width: 90%;
  max-width: 480px;
  max-height: 85vh;
  overflow-y: auto;
  padding: 1.5rem;
  box-shadow: var(--musea-shadow);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.modal-title {
  font-size: 1.125rem;
  font-weight: 700;
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--musea-text-muted);
  border-radius: var(--musea-radius-sm, 4px);
  cursor: pointer;
}

.modal-close:hover {
  background: var(--musea-border);
  color: var(--musea-text);
}

.modal-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.form-label {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--musea-text-muted);
}

.form-input {
  background: var(--musea-bg-primary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  padding: 0.5rem 0.75rem;
  color: var(--musea-text);
  font-size: 0.8125rem;
  outline: none;
  transition: border-color var(--musea-transition);
}

.form-input:focus {
  border-color: var(--musea-accent);
}

.form-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-select {
  cursor: pointer;
}

.form-select {
  option {
    background: var(--musea-bg-secondary);
    color: var(--musea-text);
  }
}

.tier-radio-group {
  display: flex;
  gap: 0.5rem;
}

.tier-radio {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  cursor: pointer;
  transition: all var(--musea-transition);
}

.tier-radio--active {
  border-color: var(--musea-accent);
  background: color-mix(in srgb, var(--musea-token-selection) 10%, transparent);
}

.tier-radio-input {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
}

.tier-radio:focus-within {
  outline: 2px solid var(--musea-accent);
  outline-offset: 2px;
}

.tier-radio-label {
  font-size: 0.8125rem;
  font-weight: 600;
}

.reference-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  max-height: 120px;
  overflow-y: auto;
  margin-top: 0.25rem;
}

.reference-option {
  font-size: 0.6875rem;
  font-family: var(--musea-font-mono);
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm, 4px);
  background: transparent;
  color: var(--musea-text-muted);
  cursor: pointer;
  transition: all var(--musea-transition);
}

.reference-option:hover {
  border-color: var(--musea-accent);
  color: var(--musea-text);
}

.reference-option--selected {
  border-color: var(--musea-accent);
  background: color-mix(in srgb, var(--musea-token-selection) 15%, transparent);
  color: var(--musea-text);
}

.form-error {
  color: var(--musea-token-error);
  font-size: 0.8125rem;
  padding: 0.5rem;
  background: color-mix(in srgb, var(--musea-token-error) 10%, transparent);
  border-radius: var(--musea-radius-md);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: var(--musea-radius-md);
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.btn--secondary {
  background: var(--musea-border);
  color: var(--musea-text);
}

.btn--secondary:hover {
  background: var(--musea-text-muted);
}

.btn--primary {
  background: var(--musea-accent);
  color: var(--musea-accent-contrast);
}

.btn--primary:hover {
  filter: brightness(1.15);
}

/* Transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active,
.modal-leave-active {
  .modal-content {
    transition: transform 0.2s ease;
  }
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from,
.modal-leave-to {
  .modal-content {
    transform: scale(0.95);
  }
}
</style>
