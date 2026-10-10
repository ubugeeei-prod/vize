import assert from "node:assert/strict";
import { compile, createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

export async function assertUsageProps(
  template: string,
  expected: Record<string, unknown>,
  componentName = "Probe",
  expectedAttrs?: Record<string, unknown>,
) {
  let received: Record<string, unknown> | undefined;
  let receivedAttrs: Record<string, unknown> | undefined;
  const Probe = defineComponent({
    inheritAttrs: expectedAttrs === undefined,
    props: Object.fromEntries(
      Object.entries(expected).map(([name, value]) => [
        name,
        typeof value === "boolean"
          ? { type: Boolean }
          : typeof value === "string"
            ? { default: "Component default" }
            : {},
      ]),
    ),
    setup(props, { attrs }) {
      received = { ...props };
      receivedAttrs = { ...attrs };
      return () => h("span", "Rendered props");
    },
  });
  const app = createSSRApp({
    components: { [componentName]: Probe },
    render: compile(template),
  });
  assert.equal(await renderToString(app), "<span>Rendered props</span>");
  assert.deepEqual(received, expected);
  assert.deepEqual(receivedAttrs, expectedAttrs ?? {});
  return received;
}
