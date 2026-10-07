// Opens screenshots in a dialog with previous/next instead of navigating to the image.
// Without JavaScript the links still open the image itself.
(function () {
  const dialog = document.querySelector(".lightbox");
  const anchors = Array.from(document.querySelectorAll("a.shot"));
  // One carousel entry per image, even when the page shows an image twice.
  const links = anchors.filter((a, i) => anchors.findIndex((b) => b.href === a.href) === i);
  if (!dialog || !links.length || typeof dialog.showModal !== "function") return;

  const img = dialog.querySelector("img");
  const caption = dialog.querySelector(".caption");
  const counter = dialog.querySelector(".counter");
  let index = 0;

  function show(i) {
    index = (i + links.length) % links.length;
    const link = links[index];
    img.src = link.href;
    img.alt = link.dataset.caption || "";
    caption.textContent = link.dataset.caption || "";
    counter.textContent = links.length > 1 ? `${index + 1} / ${links.length}` : "";
  }

  anchors.forEach((link) => {
    link.addEventListener("click", (event) => {
      event.preventDefault();
      show(links.findIndex((l) => l.href === link.href));
      dialog.showModal();
    });
  });

  dialog.querySelector(".lb-prev").addEventListener("click", () => show(index - 1));
  dialog.querySelector(".lb-next").addEventListener("click", () => show(index + 1));
  dialog.querySelector(".lb-close").addEventListener("click", () => dialog.close());

  // A click on the backdrop (the dialog itself, not its content) closes it.
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) dialog.close();
  });

  dialog.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft") show(index - 1);
    if (event.key === "ArrowRight") show(index + 1);
  });

  // Swipe on touch screens.
  let startX = null;
  dialog.addEventListener("touchstart", (event) => { startX = event.touches[0].clientX; }, { passive: true });
  dialog.addEventListener("touchend", (event) => {
    if (startX === null) return;
    const dx = event.changedTouches[0].clientX - startX;
    if (Math.abs(dx) > 40) show(index + (dx < 0 ? 1 : -1));
    startX = null;
  });

  const single = links.length < 2;
  dialog.querySelector(".lb-prev").hidden = single;
  dialog.querySelector(".lb-next").hidden = single;
})();
