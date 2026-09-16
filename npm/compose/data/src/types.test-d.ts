import {
  createDataClient,
  defineDataResource,
  type InferData,
  type InferInput,
  type InferKey,
} from "./index.ts";

const article = defineDataResource({
  key: "article.byId",
  source: "../fixtures/ArticleView.vue",
  async loader({ input }: { input: { id: string } }) {
    return {
      id: input.id,
      title: "Intro",
    };
  },
});

type ArticleInput = InferInput<typeof article>;
type ArticleData = InferData<typeof article>;
type ArticleKey = InferKey<typeof article>;

const key: ArticleKey = "article.byId";
const input: ArticleInput = { id: "intro" };
const data: ArticleData = { id: "intro", title: "Intro" };
key satisfies "article.byId";
input satisfies { id: string };
data satisfies { id: string; title: string };

const client = createDataClient();
const loaded = await client.load(article, { id: "intro" });

if (loaded.status === "success") {
  loaded.data.title satisfies string;
}

await client.load(
  article,
  { id: "intro" },
  {
    deadlineMs: 500,
    retries: 2,
    retryDelayMs(context) {
      context.key satisfies "article.byId";
      context.input satisfies { id: string };
      context.attempt satisfies number;
      context.error satisfies unknown;
      return context.attempt * 10;
    },
  },
);

createDataClient({
  sleep(milliseconds, signal) {
    milliseconds satisfies number;
    signal satisfies AbortSignal;
    return Promise.resolve();
  },
});

// @ts-expect-error missing resource input
await client.load(article, {});

defineDataResource({
  key: "broken",
  // @ts-expect-error source ownership is Vue-only
  source: "./broken.ts",
  loader: () => null,
});
