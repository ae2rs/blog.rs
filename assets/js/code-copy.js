(() => {
  const buttons = document.querySelectorAll('.code-copy-btn');
  const normalIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><path d="M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2"/></g></svg>`;
  const successIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><path d="M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2m0 9l2 2l4-4"/></g></svg>`;

  buttons.forEach((button) => {
    button.addEventListener('click', async () => {
      const container = button.closest('div');
      const code = container ? container.querySelector('pre code') : null;
      if (!code) {
        return;
      }
      const text = code.textContent || '';
      try {
        await navigator.clipboard.writeText(text);
        button.classList.add('bg-white/70', 'text-[#121212]');
        button.classList.remove('text-white/70', 'hover:text-white');
        button.innerHTML = successIcon;
        window.setTimeout(() => {
          button.classList.remove('bg-white/70', 'text-[#121212]');
          button.classList.add('text-white/70', 'hover:text-white');
          button.innerHTML = normalIcon;
        }, 1500);
      } catch (_) {
        button.classList.remove('bg-white/70', 'text-[#121212]');
        button.classList.add('text-white/70', 'hover:text-white');
        button.innerHTML = normalIcon;
        window.setTimeout(() => {
          button.innerHTML = normalIcon;
        }, 1500);
      }
    });
  });
})();
