/**
 * Mock Nuxt head management composables.
 * All are no-ops in the gallery context.
 */

interface HeadEntry {
  dispose: () => void;
  patch: (_input: Record<string, unknown>) => void;
  pause: () => void;
  resume: () => void;
}

function createHeadEntry(): HeadEntry {
  return {
    dispose: () => {},
    patch: (_input: Record<string, unknown>) => {},
    pause: () => {},
    resume: () => {},
  };
}

/**
 * Mock useHead - no-op.
 */
export function useHead(_input: Record<string, unknown>): HeadEntry {
  return createHeadEntry();
}

/**
 * Mock useSeoMeta - no-op.
 */
export function useSeoMeta(_input: Record<string, unknown>): HeadEntry {
  return createHeadEntry();
}

/**
 * Mock useHeadSafe - no-op.
 */
export function useHeadSafe(_input: Record<string, unknown>): HeadEntry {
  return createHeadEntry();
}

/**
 * Mock useServerSeoMeta - no-op.
 */
export function useServerSeoMeta(_input: Record<string, unknown>): HeadEntry {
  return createHeadEntry();
}
