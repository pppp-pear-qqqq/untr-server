export function swap(a: Element, b: Element) {
	if (a !== b) {
		const next = a.nextElementSibling;
		const parent = next !== null ? next.parentElement! : a.parentElement!;
		if (next === b) {
			parent.insertBefore(b, a);
		} else {
			b.parentElement!.insertBefore(a, b);
			parent.insertBefore(b, next);
		}
	}
}

export function bake<K extends keyof HTMLElementTagNameMap>(tagName: K, f: (e: HTMLElementTagNameMap[K]) => void) {
	const e = document.createElement(tagName);
	f(e);
	return e;
}

export function bake_tpl(tpl: HTMLTemplateElement, data: Record<string, any>): DocumentFragment;
export function bake_tpl(tpl: HTMLElement, data: Record<string, any>): HTMLElement;
export function bake_tpl(tpl: HTMLElement | HTMLTemplateElement, data: Record<string, any>) {
	const e = (tpl instanceof HTMLTemplateElement ? tpl.content.cloneNode(true) : tpl.cloneNode(true)) as HTMLElement | DocumentFragment;
	for (const [key, value] of Object.entries(data)) {
		const elems = e.querySelectorAll<HTMLElement>(`[name="${key}"],.${key}`);
		for (const e of elems) {
			if (e instanceof HTMLInputElement) {
				switch (e.type) {
					case 'checkbox':
						e.checked = value as boolean;
						break;
					case 'radio':
						e.checked = e.value === value;
						break;
					default:
						e.value = value;
				}
			} else if (e instanceof HTMLTextAreaElement) {
				e.value = value;
			} else if (e instanceof HTMLSelectElement) {
				e.value = value;
				for (const opt of e.options) opt.selected = opt.value === value;
			} else if (e.dataset.type === 'html') {
				e.innerHTML = value;
			} else {
				e.textContent = value;
			}
			e.dispatchEvent(new Event('change'));
		}
	}
	return e;
}

export async function sleep(ms: number) {
	return new Promise(resolve => setTimeout(resolve, ms));
}

export const time_formatter = new Intl.DateTimeFormat('ja-JP', {
	year: 'numeric', month: '2-digit', day: '2-digit',
	hour: '2-digit', minute: '2-digit', second: '2-digit',
	hour12: false
});
