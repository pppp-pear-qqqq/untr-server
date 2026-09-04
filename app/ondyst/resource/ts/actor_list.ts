import { Pagination } from '/common/script/pagination.js';
import { bake } from '/common/script/utils.js';
import { toast } from './toast.js';

const container = document.getElementById('actor_list') as HTMLElement;
const size = Number(document.querySelector('.pagination>.size')!.textContent);
const next = document.querySelector('.pagination>.next')!;
const limit = Number(new URLSearchParams(window.location.search).get('limit') ?? 100);

for (let i = 0; i < size / limit; ++i) {
	next.insertAdjacentHTML('beforebegin', `<a role="button" title="ページ${i + 1}" data-page="${i}" class="page">${i + 1}</a>`);
}

const page = new Pagination({ size: size, limit_default: 100, limit_max: 200 });
page.callback = (list) => {
	const fragment = document.createDocumentFragment();
	list.forEach((item) => {
		fragment.appendChild(bake('div', (e) => {
			e.classList.add('actor');
			e.dataset.id = item.id;
			e.appendChild(bake('img', (e) => {
				e.classList.add('icon');
				e.src = item.icon;
			}))
			e.appendChild(bake('a', (e) => {
				e.classList.add('name');
				e.href = `actor/${item.id}`;
				e.textContent = item.name;
			}))
			e.appendChild(bake('span', (e) => {
				e.classList.add('id');
				e.textContent = `id: ${item.id}`;
			}))
			e.appendChild(bake('p', (e) => {
				e.classList.add('comment');
				e.textContent = item.comment;
			}))
		}));
	});
	container.replaceChildren(fragment);
};
page.error = (e) => toast.error(e.message);
