// theme
document.querySelectorAll<HTMLElement>('[data-theme]:not(:root)').forEach(e => e.addEventListener('click', () => document.documentElement.dataset.theme = e.dataset.theme));
