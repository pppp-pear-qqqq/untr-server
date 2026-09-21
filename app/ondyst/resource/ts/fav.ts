import { toast } from './util/toast.js';

export const key = {
	actor: 'fav/actor',
	location: 'fav/location',
};

export const fav_actors = new Set<string>(localStorage.getItem(key.actor)?.split(','));
export const fav_locations = new Map<string, string>(JSON.parse(localStorage.getItem(key.location) ?? '[]'));

document.querySelectorAll<HTMLInputElement>('.fav>input').forEach((e) => {
	switch (e.name) {
		case 'actor': e.checked = fav_actors.has(e.value); break;
		case 'location': e.checked = fav_locations.has(e.value); break;
	}
	e.addEventListener('change', () => {
		switch (e.name) {
			case 'actor':
				if (e.checked) fav_actors.add(e.value);
				else fav_actors.delete(e.value);
				localStorage.setItem(key.actor, [...fav_actors].join(','));
				break;
			case 'location':
				if (e.checked) {
					let label: string | undefined | null = e.dataset.label;
					if (label == null || label.trim() === '') label = prompt('タブのラベルとして表示する名前を入力してください\n（あなたの画面でのみ表示されます）');
					if (label == null || label.trim() === '') {
						toast.warn('ラベルを入力してください');
						e.checked = false;
						return;
					}
					fav_locations.set(e.value, label);
				} else fav_locations.delete(e.value);
				localStorage.setItem(key.location, `[${[...fav_locations].map(([k, v]) => `["${k}","${v}"]`).join(',')}]`);
				break;
		}
	});
})
