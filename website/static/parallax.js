// Subtle parallax: elements with data-parallax="<speed>" drift against the scroll by
// <speed> times their distance from the middle of the viewport. In the hero, which starts
// on screen, they drift by <speed> times the scroll distance instead. The hero screenshot also
// tilts flat as it scrolls into view. Off for people who prefer reduced motion.
(function () {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  const layers = Array.from(document.querySelectorAll("[data-parallax]")).map((el) => ({
    el,
    speed: parseFloat(el.dataset.parallax),
    inHero: !!el.closest(".hero"),
  }));
  const tilt = document.querySelector(".hero-shot");
  let queued = false;

  function update() {
    queued = false;
    const middle = window.innerHeight / 2;
    for (const { el, speed, inHero } of layers) {
      const rect = el.parentElement.getBoundingClientRect();
      if (rect.bottom < -200 || rect.top > window.innerHeight + 200) continue;
      // Distance from the middle of the window, capped so tall sections do not drift far.
      const distance = Math.max(-middle, Math.min(middle, rect.top + rect.height / 2 - middle));
      const offset = inHero ? window.scrollY * speed : distance * speed;
      el.style.setProperty("--py", `${offset.toFixed(1)}px`);
    }
    if (tilt) {
      // 1 while the screenshot sits low in the window, 0 once it reaches the upper third.
      const top = tilt.getBoundingClientRect().top;
      const progress = Math.min(1, Math.max(0, (top - window.innerHeight * 0.25) / (window.innerHeight * 0.6)));
      tilt.style.setProperty("--tilt", progress.toFixed(3));
    }
  }

  function queue() {
    if (!queued) {
      queued = true;
      requestAnimationFrame(update);
    }
  }

  // Reveal: cards and pictures rise into place the first time they scroll into view.
  // Each group staggers per row: [selector, items per row].
  const groups = [[".pillars article", 3], [".clouds li", 8], [".features .card", 5], [".gallery figure", 4]];
  if ("IntersectionObserver" in window) {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            entry.target.classList.add("in");
            observer.unobserve(entry.target);
          }
        }
      },
      { rootMargin: "0px 0px -8% 0px" },
    );
    for (const [selector, perRow] of groups) {
      document.querySelectorAll(selector).forEach((el, i) => {
        el.classList.add("reveal");
        el.style.setProperty("--i", i % perRow);
        observer.observe(el);
      });
    }
  }

  document.documentElement.classList.add("parallax");
  window.addEventListener("scroll", queue, { passive: true });
  window.addEventListener("resize", queue);
  update();
})();
