type ToastConstructor = {
	hor?: 'left' | 'center' | 'right',
	ver?: 'top' | 'bottom',
};
type Options = {
	class_list?: [string];
	icon?: string,
	callback?: Function,
	duration?: number,
	click_dismiss?: boolean,
	color?: string,
	style?: string,
}
export class Toast {
	public static types: Record<string, Options>;

	private toast: HTMLElement;
	private default_options: Options;

	constructor({ hor = 'right', ver = 'bottom' }: ToastConstructor, default_options?: Options) {
		if (Toast.types == undefined) throw new Error('Toast.types is not defined');
		// メソッド定義
		for (const [name, options] of Object.entries(Toast.types)) {
			(this as any)[name] = (c: string | Node, ops?: Options) => this.new(name, c, merge(options, ops));
		}

		// オプション
		this.default_options = merge({
			duration: 3000,
			click_dismiss: true,
		}, default_options);
		// コンテナを作成
		this.toast = document.createElement('div');
		this.toast.className = 'notice-toast';
		this.toast.dataset.hor = hor;
		this.toast.dataset.ver = ver;
		document.body.appendChild(this.toast);
	}

	new(type: string, content: string | Node, options?: Options): ToastElement {
		const ops = merge(this.default_options, options);
		const elem = document.createElement('toast-elem') as ToastElement;
		// クラス
		elem.dataset.type = type;
		if (ops.class_list) elem.classList.add(...ops.class_list);
		// スタイル
		if (ops.style) elem.style = ops.style;
		// 背景色
		if (ops.color) elem.style.backgroundColor = ops.color;
		// アイコン
		if (ops.icon) elem.icon = ops.icon;
		// 内容
		elem.content = content;
		// コールバック
		if (ops.callback) elem.callback = ops.callback;
		// 削除準備
		if (ops.duration && ops.duration > 0) elem.duration = ops.duration;
		if (ops.click_dismiss) elem.addEventListener('click', () => {
			elem.dismiss();
		});
		// 追加
		return this.toast.appendChild(elem);
	}

	success(content: string | Node, options?: Options): ToastElement {
		return this.new('success', content, options);
	}
	info(content: string | Node, options?: Options): ToastElement {
		return this.new('info', content, options);
	}
	warn(content: string | Node, options?: Options): ToastElement {
		return this.new('warn', content, options);
	}
	error(content: string | Node, options?: Options): ToastElement {
		return this.new('error', content, options);
	}
}

function merge<T extends object>(lhs: T, rhs?: T) {
	let ret = { ...lhs };
	for (const key in rhs) if (rhs[key] != undefined) ret[key] = rhs[key];
	return ret;
}

class ToastElement extends HTMLElement {
	private _callback?: Function;
	private _dismiss_handle?: number;

	rect?: DOMRect;

	constructor() {
		super();
	}
	connectedCallback() {
		requestAnimationFrame(() => this.classList.add('show'));
		this.rect = this.getBoundingClientRect();
	}
	disconnectedCallback() {
		this._callback?.();
	}

	set type(key: string) {
		if (key in Toast.types) {
			this.dataset.type = key;
			if (Toast.types[key].color) this.style.backgroundColor = Toast.types[key].color;
			if (Toast.types[key].icon) this.icon = Toast.types[key].icon;
			if (Toast.types[key].callback) this.callback = Toast.types[key].callback;
			if (Toast.types[key].duration) this.duration = Toast.types[key].duration;
		} else {
			throw new Error('定義されていないトースト通知タイプ');
		}
	}

	set content(value: string | Node) {
		let e;
		if (typeof value === 'string') {
			e = document.createElement('div');
			e.innerText = value;
		} else if (value instanceof Element) {
			e = value;
		} else {
			e = document.createElement('div');
			e.append(value);
		}
		e.classList.add('content');
		const old = this.querySelector(':scope>.content');
		if (old) old.replaceWith(e);
		else this.appendChild(e);
	}
	set icon(value: string | HTMLOrSVGImageElement) {
		let e;
		if (value instanceof HTMLImageElement || value instanceof SVGImageElement) {
			e = value;
		} else {
			e = document.createElement('ic');
			e.classList.add(value);
		}
		e.classList.add('icon');
		const old = this.querySelector(':scope>.icon');
		if (old) old.replaceWith(e);
		else this.insertBefore(e, null);
	}
	set callback(value: Function) {
		this._callback = value;
	}
	set duration(value: number) {
		if (this._dismiss_handle) clearTimeout(this._dismiss_handle);
		if (value > 0) this._dismiss_handle = setTimeout(() => this._remove(), value);
	}

	dismiss() {
		if (this._dismiss_handle) clearTimeout(this._dismiss_handle);
		this._remove();
	}

	private _remove() {
		this.classList.remove('show');
		setTimeout(() => this.remove(), 120);
	}
}
customElements.define('toast-elem', ToastElement);
