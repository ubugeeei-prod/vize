
const Styled = ({ color, gap }) => (
  <article
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>
    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </article>
);
