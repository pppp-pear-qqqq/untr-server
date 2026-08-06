// theme
document.querySelectorAll<HTMLElement>('[data-theme]:not(:root)').forEach(e => e.addEventListener('click', () => {
	document.documentElement.dataset.theme = e.dataset.theme!;
	localStorage.setItem('theme', e.dataset.theme!);
}));

// toggle aria-expanded
document.querySelectorAll<HTMLElement>('[aria-expanded]').forEach(e => {
	e.addEventListener('click', () => e.setAttribute('aria-expanded', e.getAttribute('aria-expanded') !== 'true' ? 'true' : 'false'))
})

// password visibility
document.querySelectorAll<HTMLElement>('input[type="password"]+.toggle-visible').forEach(e => {
	const input = e.previousElementSibling as HTMLInputElement;
	e.addEventListener('click', () => {
		input.type = input.type === 'password' ? 'text' : 'password';
	});
});

// help dialog
const dialog = document.getElementById('help') as HTMLDialogElement;
document.querySelectorAll<HTMLElement>('.help').forEach(e => {
	e.addEventListener('click', () => {
		dialog.innerHTML = e.innerHTML;
		dialog.showModal();
	});
});
dialog.addEventListener('close', () => {
	dialog.innerHTML = '';
});
