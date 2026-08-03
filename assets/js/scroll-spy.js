(() => {
  const headings = Array.from(document.querySelectorAll('main h1[id], main h2[id], main h3[id]'));
  if (headings.length === 0) {
    return;
  }

  // Matches the `scroll-mt-24` utility on rendered headings. The root font-size is a
  // clamp(), so this is deliberately approximate.
  const ACTIVATION_OFFSET = 96;
  const SETTLE_DELAY = 150;

  let written = window.location.hash.slice(1);
  let frame = 0;
  let timer = 0;

  const activeHeading = () => {
    const scrolled = window.scrollY + window.innerHeight;
    if (scrolled >= document.documentElement.scrollHeight - 2) {
      return headings[headings.length - 1];
    }
    let active = null;
    for (const heading of headings) {
      if (heading.getBoundingClientRect().top > ACTIVATION_OFFSET + 1) {
        break;
      }
      active = heading;
    }
    return active;
  };

  const commit = () => {
    const heading = activeHeading();
    const id = heading ? heading.id : '';
    if (id === written) {
      return;
    }
    written = id;
    const url = window.location.pathname + window.location.search + (id ? '#' + id : '');
    window.history.replaceState(window.history.state, '', url);
  };

  const onScroll = () => {
    if (frame) {
      return;
    }
    frame = window.requestAnimationFrame(() => {
      frame = 0;
      // Only rewrite the URL once scrolling settles: smooth-scrolled anchor jumps sweep
      // past many headings, and committing each one churns history.replaceState.
      window.clearTimeout(timer);
      timer = window.setTimeout(commit, SETTLE_DELAY);
    });
  };

  // The script is deferred, so it runs before the browser has honoured an incoming
  // fragment or restored the previous scroll position. Wait for both to settle before
  // touching the URL, otherwise the first update would see scrollY 0 and drop the hash.
  window.addEventListener('load', () => {
    window.requestAnimationFrame(() => {
      window.addEventListener('scroll', onScroll, { passive: true });
      window.addEventListener('resize', onScroll, { passive: true });
    });
  });
})();
