type RequestMethod = 'GET' | 'HEAD' | 'QUERY' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
type ResponseType = 'arrayBuffer' | 'blob' | 'bytes' | 'formData' | 'json' | 'text';
type EncodeType = 'application/x-www-form-urlencoded' | 'form' | 'multipart/form-data' | 'multipart' | 'application/json' | 'json';

export interface HttpRequestOptions {
	query?: string | Record<string, string> | URLSearchParams | string[][];
	body?: FormData | HTMLFormElement | Record<string, any>;
	encode?: EncodeType;
	ret_type?: ResponseType;
	headers?: HeadersInit;
}

export class HttpClient {
	private actions = new Set<string>();
	public wait: number;

	constructor(wait: number = 1000) {
		this.wait = wait;
	}

	/**
	 * HTMLフォームから直接リクエストを送信
	 */
	public send(form: HTMLFormElement): Promise<Response>;
	public send(form: HTMLFormElement, ret_type: 'arrayBuffer'): Promise<ArrayBuffer>;
	public send(form: HTMLFormElement, ret_type: 'blob'): Promise<Blob>;
	public send(form: HTMLFormElement, ret_type: 'bytes'): Promise<Uint8Array>;
	public send(form: HTMLFormElement, ret_type: 'formData'): Promise<FormData>;
	public send<T = any>(form: HTMLFormElement, ret_type: 'json'): Promise<T>;
	public send(form: HTMLFormElement, ret_type: 'text'): Promise<string>;
	public send(form: HTMLFormElement, ret_type?: ResponseType): Promise<any> {
		if (!form.checkValidity()) return Promise.reject(new Error('フォームの制約が満たされていません'));

		const url = form.action;
		const method = (form.method || 'get').toUpperCase() as 'GET' | 'POST';

		const enctype = (form.getAttribute('enctype') || form.enctype) as EncodeType;

		switch (method) {
			case 'GET': return this.request(url, 'GET', {
				query: new URLSearchParams(new FormData(form) as any),
				ret_type
			});
			case 'POST': return this.request(url, 'POST', {
				body: form,
				encode: enctype,
				ret_type
			});
		}
	}

	public get(url: string, query?: HttpRequestOptions['query']): Promise<Response>;
	public get(url: string, query: HttpRequestOptions['query'] | null | undefined, ret_type: 'arrayBuffer'): Promise<ArrayBuffer>;
	public get(url: string, query: HttpRequestOptions['query'] | null | undefined, ret_type: 'blob'): Promise<Blob>;
	public get(url: string, query: HttpRequestOptions['query'] | null | undefined, ret_type: 'bytes'): Promise<Uint8Array>;
	public get(url: string, query: HttpRequestOptions['query'] | null | undefined, ret_type: 'formData'): Promise<FormData>;
	public get<T = any>(url: string, query: HttpRequestOptions['query'] | null | undefined, ret_type: 'json'): Promise<T>;
	public get(url: string, query: HttpRequestOptions['query'] | null | undefined, ret_type: 'text'): Promise<string>;
	public get(url: string, query?: HttpRequestOptions['query'], ret_type?: ResponseType): Promise<any> {
		return this.request(url, 'GET', { query, ret_type });
	}

	public post(url: string, data?: HttpRequestOptions['body'], encode?: EncodeType): Promise<Response>;
	public post(url: string, data: HttpRequestOptions['body'] | null | undefined, encode: EncodeType | null | undefined, ret_type: 'arrayBuffer'): Promise<ArrayBuffer>;
	public post(url: string, data: HttpRequestOptions['body'] | null | undefined, encode: EncodeType | null | undefined, ret_type: 'blob'): Promise<Blob>;
	public post(url: string, data: HttpRequestOptions['body'] | null | undefined, encode: EncodeType | null | undefined, ret_type: 'bytes'): Promise<Uint8Array>;
	public post(url: string, data: HttpRequestOptions['body'] | null | undefined, encode: EncodeType | null | undefined, ret_type: 'formData'): Promise<FormData>;
	public post<T = any>(url: string, data: HttpRequestOptions['body'] | null | undefined, encode: EncodeType | null | undefined, ret_type: 'json'): Promise<T>;
	public post(url: string, data: HttpRequestOptions['body'] | null | undefined, encode: EncodeType | null | undefined, ret_type: 'text'): Promise<string>;
	public post(url: string, data: HttpRequestOptions['body'], encode: EncodeType = 'application/json', ret_type?: ResponseType): Promise<any> {
		return this.request(url, 'POST', { body: data, encode, ret_type });
	}

	/**
	 * 汎用リクエストメソッド
	 */
	public request(method: string, url: string, options?: Omit<HttpRequestOptions, 'ret_type'>): Promise<Response>;
	public request(method: string, url: string, options: HttpRequestOptions & { ret_type: 'arrayBuffer' }): Promise<ArrayBuffer>;
	public request(method: string, url: string, options: HttpRequestOptions & { ret_type: 'blob' }): Promise<Blob>;
	public request(method: string, url: string, options: HttpRequestOptions & { ret_type: 'bytes' }): Promise<Uint8Array>;
	public request(method: string, url: string, options: HttpRequestOptions & { ret_type: 'formData' }): Promise<FormData>;
	public request<T = any>(method: string, url: string, options: HttpRequestOptions & { ret_type: 'json' }): Promise<T>;
	public request(method: string, url: string, options: HttpRequestOptions & { ret_type: 'text' }): Promise<string>;
	public request(url: string, method: RequestMethod, options: HttpRequestOptions = {}): Promise<any> {
		const action_key = `${url}:${method}`;
		if (this.actions.has(action_key)) return Promise.reject(new Error('既に同リクエストを処理中です'));

		const req: RequestInit = { method, headers: options.headers };
		const full_url = options.query ? `${url}?${new URLSearchParams(options.query).toString()}` : url;

		// POSTなどの場合、bodyの構築
		if (options.body && method !== 'GET' && method !== 'HEAD') {
			this.build_body(req, options.body, options.encode || 'application/json');
		}

		this.actions.add(action_key);
		setTimeout(() => this.actions.delete(action_key), this.wait);

		return fetch(full_url, req).then(r => {
			if (!r.ok) {
				return r.text().then(e => {
					console.error(e);
					throw new Error(e);
				});
			}

			switch (options.ret_type) {
				case 'arrayBuffer': return r.arrayBuffer();
				case 'blob': return r.blob();
				case 'bytes': return r.bytes();
				case 'formData': return r.formData();
				case 'json': return r.json();
				case 'text': return r.text();
				default: return r; // 指定がない場合は Response オブジェクトをそのまま返す
			}
		});
	}

	/**
	 * bodyとヘッダーをエンコード形式に従って構築します
	 */
	private build_body(req: RequestInit, data: FormData | HTMLFormElement | Record<string, any>, encode: EncodeType): void {
		switch (encode) {
			case 'multipart/form-data':
			case 'multipart': {
				// FormDataをそのまま渡す
				if (data instanceof HTMLFormElement) {
					req.body = new FormData(data);
				} else if (data instanceof FormData) {
					req.body = data;
				} else {
					// Record等の場合はFormDataに変換
					const fd = new FormData();
					for (const [key, value] of Object.entries(data)) {
						if (value != null) fd.append(key, value instanceof Blob ? value : String(value));
					}
					req.body = fd;
				}
			} break;
			case 'application/x-www-form-urlencoded':
			case 'form': {
				if (data instanceof FormData || data instanceof HTMLFormElement) {
					req.body = new URLSearchParams(new FormData(data as any) as any);
				} else {
					req.body = new URLSearchParams(
						Object.entries(data)
							.filter(([_, value]) => value != null)
							.map(([k, v]) => [k, String(v)])
					);
				}
			} break;
			case 'application/json':
			case 'json': {
				// Content-Type設定
				(req.headers as Record<string, string>)['Content-Type'] = 'application/json';

				if (data instanceof FormData) {
					const buf: Record<string, any> = {};
					for (const [key, value] of data.entries()) {
						if (Object.prototype.hasOwnProperty.call(buf, key)) {
							// キーが重複していたら配列化
							if (!Array.isArray(buf[key])) buf[key] = [buf[key]];
							buf[key].push(value); // 値を追加
						} else buf[key] = value;
					}
					req.body = JSON.stringify(buf);
				} else if (data instanceof HTMLFormElement) {
					const buf: Record<string, any> = {};
					for (const e of data.elements) {
						const name = e.getAttribute('name');
						if (name && 'value' in e) {
							const el = e as HTMLInputElement | HTMLTextAreaElement | HTMLButtonElement;

							// 空文字かつ許可されていなければスキップ
							if (el.value === '' && el.dataset.allowEmpty !== 'true') continue;

							switch (el.type) {
								case 'number': buf[name] = Number(el.value); break;
								case 'checkbox': buf[name] = (el as HTMLInputElement).checked; break;
								case 'radio': if ((el as HTMLInputElement).checked) buf[name] = el.value; break;
								default: switch (el.dataset.type) {
									case 'number': buf[name] = Number(el.value); break;
									case 'boolean': buf[name] = (el as HTMLInputElement).checked || el.value === 'true'; break;
									default: buf[name] = el.value;
								}
							}
						}
					}
					req.body = JSON.stringify(buf);
				} else {
					req.body = JSON.stringify(data);
				}
			} break;
		}
	}
}
