import { Ajax } from '/common/script/ajax.js';
import { toast } from './import.js';

let data: Record<string, string> = {};
const form = document.querySelector('form')!;
form.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('[name]').forEach(e => {
	e.addEventListener('change', () => {
		e.classList.add('changed');
		data[e.name] = e.value;
	})
});
form.addEventListener('submit', async (ev) => {
	ev.preventDefault();
	const form = ev.currentTarget as HTMLFormElement;
	try {
		await new Ajax(form.action).method('PATCH').body(data, 'json').send();
		toast.success('更新しました');
	} catch (err: any) {
		toast.error(err.message);
	}
});
