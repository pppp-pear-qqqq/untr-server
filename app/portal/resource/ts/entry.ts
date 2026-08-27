import { Ajax } from '/common/script/ajax.js';
import { sleep } from '/common/script/utils.js';
import { toast } from './import.js';

document.querySelectorAll('form').forEach(e => {
	e.addEventListener('submit', async (ev) => {
		ev.preventDefault()
		try {
			await new Ajax(e).send();
			switch (e.id) {
				case 'login': toast.success('ログインしました'); break;
				case 'register': toast.success('新規登録が完了しました'); break;
			}
			await sleep(2000);
			location.href = 'home';
		} catch (err: any) {
			toast.error(err.message);
		}
	})
})
