(() => {
  const initializeVeinBackground = () => {
    if (typeof THREE === "undefined") {
      console.warn("[vein] Three.js not loaded");
      return;
    }

    const canvas = document.getElementById("vein-canvas");
    if (!canvas) return;

    const renderer = new THREE.WebGLRenderer({
      canvas,
      alpha: true,
      premultipliedAlpha: false,
      antialias: false,
      powerPreference: "high-performance",
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.5));

    const scene = new THREE.Scene();
    const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
    const uniforms = {
      u_resolution: { value: new THREE.Vector2() },
      u_time: { value: 0.0 },
      u_scroll: { value: 0.0 },
      u_dark: { value: 0.0 },
      u_mouse: { value: new THREE.Vector2(0.5, 0.5) },
      u_mouseVel: { value: 0.0 },
    };

    const material = new THREE.ShaderMaterial({
      vertexShader: `__VERT_SRC__`,
      fragmentShader: `__FRAG_SRC__`,
      uniforms,
      transparent: true,
      depthWrite: false,
    });

    scene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), material));

    let mouseX = 0.5;
    let mouseY = 0.5;
    let targetX = 0.5;
    let targetY = 0.5;
    let mouseVel = 0.0;
    let prevMX = 0.5;
    let prevMY = 0.5;

    const updatePointer = (clientX, clientY) => {
      targetX = clientX / window.innerWidth;
      targetY = 1.0 - clientY / window.innerHeight;
    };

    document.addEventListener(
      "mousemove",
      (event) => {
        updatePointer(event.clientX, event.clientY);
      },
      { passive: true },
    );

    document.addEventListener(
      "touchmove",
      (event) => {
        if (event.touches.length === 0) return;
        updatePointer(event.touches[0].clientX, event.touches[0].clientY);
      },
      { passive: true },
    );

    let scrollY = window.scrollY;
    const header = document.querySelector(".header");

    window.addEventListener(
      "scroll",
      () => {
        scrollY = window.scrollY;
        if (!header || !document.body.classList.contains("entry-page")) return;

        if (scrollY > 100) {
          header.classList.add("header-visible");
          return;
        }

        header.classList.remove("header-visible");
      },
      { passive: true },
    );

    const isDark = () => {
      const theme = document.documentElement.getAttribute("data-theme");
      if (theme === "dark") return true;
      if (theme === "light") return false;
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    };

    const resize = () => {
      const width = canvas.clientWidth || window.innerWidth;
      const height = canvas.clientHeight || window.innerHeight;

      renderer.setSize(width, height, false);
      uniforms.u_resolution.value.set(canvas.width, canvas.height);
    };

    window.addEventListener("resize", resize);
    resize();

    const clock = new THREE.Clock();
    let elapsedTime = 0.0;

    const render = () => {
      requestAnimationFrame(render);

      const dt = clock.getDelta();
      elapsedTime += dt;
      uniforms.u_time.value = elapsedTime;
      uniforms.u_scroll.value = scrollY;
      uniforms.u_dark.value = isDark() ? 1.0 : 0.0;

      mouseX += (targetX - mouseX) * 0.05;
      mouseY += (targetY - mouseY) * 0.05;

      const dx = mouseX - prevMX;
      const dy = mouseY - prevMY;
      const speed = Math.hypot(dx, dy) / Math.max(dt, 0.001);
      mouseVel += (speed - mouseVel) * 0.1;
      prevMX = mouseX;
      prevMY = mouseY;

      uniforms.u_mouse.value.set(mouseX, mouseY);
      uniforms.u_mouseVel.value = mouseVel;

      renderer.render(scene, camera);
    };

    render();
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializeVeinBackground, {
      once: true,
    });
    return;
  }

  initializeVeinBackground();
})();
