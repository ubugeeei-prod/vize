import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import App from "../components/组件 space/FixtureCounter.vue";
import Client from "./client.jsx";

export default async function Page() {
  const html = await renderToString(createSSRApp(App, { initial: 2 }));
  return (
    <>
      <Client html={html} />
      <aside id="outside" className="counter">
        outside
      </aside>
    </>
  );
}
