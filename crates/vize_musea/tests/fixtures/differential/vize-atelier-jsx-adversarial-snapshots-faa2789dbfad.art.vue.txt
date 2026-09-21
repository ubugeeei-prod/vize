
const Styled = ({ color, gap }) => (
  <article class="box">
    <p>content</p>
    <style scoped>{`
      .box {
        color: ${color};
        gap: ${gap}px;
      }
    `}</style>
  </article>
);
