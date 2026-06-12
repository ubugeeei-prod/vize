const rows = [
  { id: "portrait", label: "Portrait" },
  { id: "caption", label: "Caption" },
];

const AccessibleMedia = () => (
  <article class="media-card">
    <img src="/media/team-avatar.png" alt="Avatar for Ada Lovelace" />
    <h2>Ada Lovelace</h2>

    <ul>
      {rows.map((row) => (
        <li key={row.id}>{row.label}</li>
      ))}
    </ul>
  </article>
);

export default AccessibleMedia;
