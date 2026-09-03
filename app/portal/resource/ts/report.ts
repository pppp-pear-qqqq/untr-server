import { Ajax } from '/common/script/ajax.js';
import { toast } from './toast.js';

if (location.hash) {
	const e = document.querySelector<HTMLOptionElement>(`[name="app_name"]>[value="${location.hash.substring(1)}"]`);
	if(e) e.selected = true;
}

document.querySelector('form')!.addEventListener('submit', async (ev) => {
	ev.preventDefault();
	const form = ev.currentTarget as HTMLFormElement;
	try {
		await new Ajax(form).send();
		toast.success('報告が完了しました');
	} catch (err: any) {
		toast.error(err.message);
	}
});
