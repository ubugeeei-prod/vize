window.__backgroundMotion = { frames: [], gpuDraws: 0 };
for (const Context of [WebGLRenderingContext, WebGL2RenderingContext]) {
  for (const name of ["drawArrays", "drawElements"]) {
    if (!Object.hasOwn(Context.prototype, name)) continue;
    const draw = Context.prototype[name];
    Context.prototype[name] = function (...args) {
      const result = draw.apply(this, args);
      window.__backgroundMotion.gpuDraws++;
      return result;
    };
  }
}
const Renderer = THREE.WebGLRenderer;
THREE.WebGLRenderer = function (...args) {
  const renderer = new Renderer(...args);
  const render = renderer.render;
  renderer.render = function (scene, camera) {
    const result = render.call(this, scene, camera);
    const u = scene.children[0].material.uniforms;
    window.__backgroundMotion.frames.push({
      time: u.u_time.value,
      scroll: u.u_scroll.value,
      dark: u.u_dark.value,
      mouse: [u.u_mouse.value.x, u.u_mouse.value.y],
      velocity: u.u_mouseVel.value,
    });
    return result;
  };
  return renderer;
};
