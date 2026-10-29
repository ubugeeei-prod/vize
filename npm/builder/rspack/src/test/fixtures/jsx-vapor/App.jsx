// Standalone Vapor renderers currently support roots without captured bindings.
const _App = () => (
  <div class="jsx-vapor-app">
    <p>hello vapor jsx</p>
    <span>{1 + 1}</span>
  </div>
);

export default _App;
