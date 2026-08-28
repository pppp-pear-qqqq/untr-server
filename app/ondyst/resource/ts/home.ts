import { Ajax } from '/common/script/ajax.js';
import { bake } from '/common/script/utils.js';
import { toast } from './toast.js';
import { fav_actors, fav_locations } from './fav.js';

const tabs = document.querySelector<HTMLElement>('#timeline>.tabs')!;
for (const location of fav_locations) {
	const item = location.split(':');
	tabs.appendChild(bake('label', (e) => {
		e.classList.add('tab');
		e.role = 'button';
		e.append(
			bake('input', (e) => {
				e.type = 'radio';
				e.name = 'tab';
				e.value = item[0];
			}),
			document.createTextNode(item[1]),
		);
	}));
}
