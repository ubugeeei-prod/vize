import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { computed, defineComponent, h } from 'vue'
import type { CompilerOptions, SfcCompileResult, WasmModule } from '../src/wasm/index'

let currentCompiler: WasmModule | null = null

vi.mock('../src/wasm/index', () => ({
  getWasm: () => currentCompiler,
  loadWasm: vi.fn(),
}))

vi.mock('../src/shared/MonacoEditor.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: {
        type: String,
        default: '',
      },
    },
    emits: ['update:modelValue'],
    setup(props, { emit }) {
      return () =>
        h('textarea', {
          class: 'monaco-editor-stub',
          value: props.modelValue,
          onInput: (event: Event) => {
            emit('update:modelValue', (event.target as HTMLTextAreaElement).value)
          },
        })
    },
  }),
}))

vi.mock('../src/shared/CodeHighlight.vue', () => ({
  default: defineComponent({
    props: {
      code: {
        type: String,
        default: '',
      },
    },
    setup(props) {
      return () => h('pre', { class: 'code-highlight-stub' }, props.code)
    },
  }),
}))

import AtelierPlayground from '../src/features/atelier/AtelierPlayground.vue'

function createSfcResult(
  scriptCode: string,
  templateCode: string,
  templates: string[] = [],
): SfcCompileResult {
  return {
    descriptor: {
      filename: 'example.vue',
      source: '',
      styles: [],
      customBlocks: [],
      template: {
        content: '<div />',
        loc: { start: 0, end: 0 },
        attrs: {},
      },
      scriptSetup: {
        content: '',
        loc: { start: 0, end: 0 },
        attrs: {},
        lang: 'ts',
        setup: true,
      },
    },
    script: {
      code: scriptCode,
      bindings: {
        bindings: {},
      },
    },
    template: {
      code: templateCode,
      preamble: '',
      ast: {},
      helpers: [],
      templates,
    },
    css: '',
    errors: [],
    warnings: [],
    bindingMetadata: {},
  }
}

describe('Atelier output tabs', () => {
  it('switches between VDOM, SSR, and Vapor outputs without shifting the code view toggle', async () => {
    const compileSfc = vi.fn((_: string, options: CompilerOptions) => {
      if (options.outputMode === 'vapor') {
        return createSfcResult(
          "const mode = 'dom-script'",
          "const mode = 'vapor'",
          ['const t0 = _template("<div></div>")'],
        )
      }
      if (options.ssr) {
        return createSfcResult("const mode = 'dom-script'", "const mode = 'ssr'")
      }
      return createSfcResult("const mode = 'dom'", "const mode = 'dom-template'")
    })

    currentCompiler = {
      compileSfc,
      compileCss: vi.fn(() => ({
        code: '',
        cssVars: [],
        errors: [],
        warnings: [],
      })),
    } as unknown as WasmModule

    const wrapper = mount(AtelierPlayground, {
      props: {
        compiler: currentCompiler,
      },
      global: {
        provide: {
          theme: computed(() => 'light' as const),
        },
      },
    })

    const outputHeading = () => wrapper.find('.code-header h4')
    const codeOutputs = () => wrapper.findAll('.code-highlight-stub')
    const codeViewButtons = () =>
      wrapper.findAll('.code-view-toggle .toggle-btn').map((button) => ({
        text: button.text().trim(),
        disabled: button.attributes('disabled') !== undefined,
      }))

    await vi.waitFor(() => {
      expect(outputHeading().text()).toBe('VDOM Output')
      expect(codeOutputs()).toHaveLength(1)
      expect(codeOutputs()[0]!.text()).toContain("const mode = 'dom'")
      expect(codeViewButtons()).toEqual([
        { text: 'TS', disabled: false },
        { text: 'JS', disabled: false },
      ])
    })

    const buttons = () => wrapper.findAll('button')
    const findButton = (label: string) => buttons().find((button) => button.text().trim() === label)

    await findButton('SSR')!.trigger('click')
    await vi.waitFor(() => {
      expect(outputHeading().text()).toBe('SSR Output')
      expect(codeOutputs()).toHaveLength(1)
      expect(codeOutputs()[0]!.text()).toContain("const mode = 'ssr'")
      expect(wrapper.text()).not.toContain('SFC Output')
      expect(codeViewButtons()).toEqual([
        { text: 'TS', disabled: true },
        { text: 'JS', disabled: true },
      ])
    })

    await findButton('Vapor')!.trigger('click')
    await vi.waitFor(() => {
      expect(outputHeading().text()).toBe('Vapor Output')
      expect(codeOutputs()).toHaveLength(1)
      expect(codeOutputs()[0]!.text()).toContain("const mode = 'vapor'")
      expect(wrapper.text()).not.toContain('Template Fragments')
      expect(wrapper.text()).not.toContain('SFC Output')
      expect(codeViewButtons()).toEqual([
        { text: 'TS', disabled: true },
        { text: 'JS', disabled: true },
      ])
    })

    expect(findButton('Vapor SSR')).toBeUndefined()

    wrapper.unmount()
  })
})
