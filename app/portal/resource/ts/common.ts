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

let insert_target_element: HTMLInputElement | HTMLTextAreaElement | undefined;
document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('.insert-tag').forEach(e => e.addEventListener('focusin', () => insert_target_element = e));
function insert_tag(pre: string, suf: string, elem?: HTMLInputElement | HTMLTextAreaElement) {
	if (elem ??= insert_target_element) {
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
