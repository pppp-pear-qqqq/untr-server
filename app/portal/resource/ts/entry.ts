import { Ajax } from '/common/script/ajax.js';
import { sleep } from '/common/script/funcs.js';
import { toast } from './import.js';

document.querySelectorAll('form').forEach(e => {
	e.addEventListener('submit', ev => {
		ev.preventDefault()
		new Ajax(e).send().then(async () => {
			if (e.id === 'login') toast.success('ログインしました');
			else if (e.id === 'register') toast.success('新規登録が完了しました');
			await sleep(2000);
			location.href = 'home';
		})
	})
})
