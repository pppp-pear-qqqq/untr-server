export class Ajax {
	private static actions = new Set<string>();
	private static _wait = 1000;

	static set wait(value: number) { this._wait = value }

	private _url?: string;
	private _query?: URLSearchParams;
	private _req: RequestInit;
	private _ret_type?: 'text' | 'json' | 'arrayBuffer' | 'blob' | 'bytes' | 'formData';

	constructor();
	constructor(form: HTMLFormElement);
	constructor(url: string);
	constructor(arg?: HTMLFormElement | string) {
		this._req = {};
		if (arg instanceof HTMLFormElement) {
			if (!arg.checkValidity()) throw new Error('フォームの制約が満たされていません');
			this._url = arg.action;
			if (arg.elements.length > 0) switch (arg.method) {
				case 'get': {
					this._query = new URLSearchParams(new FormData(arg) as any);
				} break;
				case 'post': {
					if (!['form', 'json'].includes(arg.dataset.encode!)) throw new Error('エンコード形式が指定されていません');
					this.post(arg, arg.dataset.encode as 'form' | 'json');
				} break;
			}
		} else {
			this._url = arg;
		}
	}

	url(url: string): Ajax {
		this._url = url;
		return this;
	}

	ret_type(ret_type: 'arrayBuffer' | 'blob' | 'bytes' | 'formData' | 'json' | 'text'): Ajax {
		this._ret_type = ret_type;
		return this;
	}

	query(data: string[][] | Record<string, string> | string | URLSearchParams): Ajax {
		this._query = new URLSearchParams(data);
		return this;
	}

	body(data: FormData | HTMLFormElement | Record<string, any>, encode: 'form' | 'json'): Ajax {
		if (!this._req) this._req = {};
		switch (encode) {
			case 'form': if (data instanceof FormData) {
				// フォームデータの場合整形済みと見做してそのまま
				this._req.body = new URLSearchParams(data as any);
			} else if (data instanceof HTMLFormElement) {
				// HTMLFormElementの場合空欄を無視する？　なんか何もしてないね
				this._req.body = new URLSearchParams(new FormData(data) as any);
			} else {
				// Recordの場合、valueが無いものを除外し、値をstringに変換する
				this._req.body = new URLSearchParams(Object.entries(data).filter(([_, value]) => value != null).map(([k, v]) => [k, String(v)]));
			} break;
			case 'json': if (data instanceof FormData) {
				const buf: Record<string, any> = {};
				for (const [key, value] of data.entries()) {
					if (Object.prototype.hasOwnProperty.call(buf, key)) {
						if (!Array.isArray(buf[key])) buf[key] = [buf[key]];
						buf[key].push(value);
					} else buf[key] = value;
				}
				this._req.body = JSON.stringify(buf);
			} else if (data instanceof HTMLFormElement) {
				const buf: Record<string, any> = {};
				for (const e of data.elements) {
					const name = e.getAttribute('name');
					if (name && 'value' in e && (e.value || ('minLength' in e && e.minLength !== -1))) {
						switch ((e as (HTMLInputElement | HTMLTextAreaElement | HTMLButtonElement)).type) {
							case 'number': buf[name] = Number(e.value); break;
							case 'checkbox': buf[name] = (e as HTMLInputElement).checked; break;
							case 'radio': if ((e as HTMLInputElement).checked) buf[name] = e.value; break;
							default: switch ((e as unknown as HTMLElement).dataset.type) {
								case 'number': buf[name] = Number(e.value); break;
								case 'boolean': buf[name] = (e as HTMLInputElement).checked || e.value === 'true'; break;
								default: buf[name] = e.value;
							};
						}
					}
				}
				this._req.body = JSON.stringify(buf);
			} else {
				// Recordの場合そのまま
				this._req.body = JSON.stringify(data);
			} this._req.headers = { 'Content-Type': 'application/json' }; break;
		}
		console.log(this._req.body);
		return this;
	}

	method(method: 'GET' | 'HEAD' | 'POST' | 'PUT' | 'DELETE' | 'CONNECT' | 'OPTIONS' | 'TRACE' | 'PATCH'): Ajax {
		this._req.method = method;
		return this;
	}

	get(data: string[][] | Record<string, string> | string | URLSearchParams): Ajax {
		return this.query(data).method('GET');
	}

	post(data: FormData, encode: 'form'): Ajax;
	post(data: HTMLFormElement, encode?: 'form' | 'json'): Ajax;
	post(data: Record<string, any>, encode?: 'form' | 'json'): Ajax;
	post(data: FormData | HTMLFormElement | Record<string, any>, encode: 'form' | 'json' = 'json'): Ajax {
		return this.body(data, encode).method('POST');
	}

	send(): Promise<Response>;
	send(ret_type: 'arrayBuffer'): Promise<ArrayBuffer>;
	send(ret_type: 'blob'): Promise<Blob>;
	send(ret_type: 'bytes'): Promise<Uint8Array>;
	send(ret_type: 'formData'): Promise<FormData>;
	send(ret_type: 'json'): Promise<any>;
	send(ret_type: 'text'): Promise<string>;
	send(ret_type?: 'arrayBuffer' | 'blob' | 'bytes' | 'formData' | 'json' | 'text'): Promise<any> {
		if (!this._url) throw new Error('URL未設定');
		if (Ajax.actions.has(this._url)) throw new Error('既に同リクエストを処理中です');
		Ajax.actions.add(this._url);
		setTimeout(() => Ajax.actions.delete(this._url!), Ajax._wait);
		return fetch(this._query ? `${this._url}?${this._query.toString()}` : this._url, this._req)
			.then(r => {
				if (r.ok) {
					switch (ret_type ? ret_type : this._ret_type) {
						case 'arrayBuffer': return r.arrayBuffer();
						case 'blob': return r.blob();
						case 'bytes': return r.bytes();
						case 'formData': return r.formData();
						case 'json': return r.json();
						case 'text': return r.text();
						default: return r;
					}
				} else return r.text().then(e => {
					console.error(e);
					throw new Error(e);
				});
			})
	}
}
