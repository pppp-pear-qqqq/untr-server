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

// insert tag
let insert_target: HTMLInputElement | HTMLTextAreaElement;
document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('.insert-tag').forEach(e => {
	e.addEventListener('focusin', () => insert_target = e);
	if (!insert_target) insert_target = e;
});
window.insert_tag = function (pre: string, suf: string, elem?: HTMLInputElement | HTMLTextAreaElement) {
	if (elem ??= insert_target) {
		const start = elem.selectionStart, end = elem.selectionEnd;
		if (start != null && end != null) {
			const prev = elem.value;
			elem.value = prev.slice(undefined, start) + pre + prev.slice(start, end) + suf + prev.slice(end);
			elem.selectionStart = start + pre.length;
			elem.selectionEnd = end + pre.length;
			elem.dispatchEvent(new Event('change'));
			elem.focus();
		}
	}
}
