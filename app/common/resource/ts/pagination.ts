import { Ajax } from './ajax.js';
import { clamp } from './utils.js';

type Options = {
	size: number;
	limit_default: number;
	limit_max: number;
};

export class Pagination {
	private size: number;
	private limit_default: number;
	private limit_max: number;
	private _callback: ((list: any[]) => void) | undefined;
	private _error: ((e: Error) => void) | undefined;

	private search: URLSearchParams;

	constructor({ size, limit_default, limit_max }: Options) {
		this.size = size;
		this.limit_default = limit_default;
		this.limit_max = limit_max;
		this.search = new URLSearchParams(location.search);

		// イベントをセット
		document.querySelectorAll<HTMLElement>('.pagination :is([data-step],[data-page])').forEach(e => {
			e.addEventListener('click', () => {
				// この関数内では数値の正規化は行わない
				const offset = Number(this.search.get('offset') ?? 0);
				const limit = Number(this.search.get('limit') ?? this.limit_default);

				let target = offset;
				if (e.dataset.step) {
					target = Number(e.dataset.step) * limit + offset;
				} else if (e.dataset.page) {
					if (e.dataset.page === 'last') {
						target = Math.max((Math.ceil(this.size / limit) - 1), 0) * limit;
					} else {
						target = Number(e.dataset.page) * limit;
					}
				}

				this.reload(target, limit);
			});
		});
		// URLが戻った時に、クラス内の状態も更新して再フェッチする
		window.addEventListener('popstate', () => {
			const search = new URLSearchParams(location.search);
			const offset = Number(search.get('offset') ?? 0);
			const limit = Number(search.get('limit') ?? this.limit_default);
			this.reload(offset, limit, false);
		});
	}

	set callback(value: (list: any[]) => void) {
		this._callback = value;
	}
	set error(value: (e: Error) => void) {
		this._error = value;
	}

	/// 要素を再読み込み
	public async reload(offset?: number, limit?: number, push_history: boolean = true) {
		// 数値の正規化
		const l = clamp(limit == null || isNaN(limit) ? this.limit_default : limit, 1, this.limit_max);
		const o = clamp(offset == null || isNaN(offset) ? 0 : offset, 0, Math.max((Math.ceil(this.size / l) - 1) * l, 0));
		// 変更を判定
		const diff = Number(this.search.get('offset') ?? 0) !== o || Number(this.search.get('limit') ?? this.limit_default) !== l;
		// if (!diff) return null;
		// クエリパラメータを更新
		this.search.set('offset', o.toString());
		this.search.set('limit', l.toString());
		// API呼び出し
		try {
			const ret: { size: number; list: any[] } = await new Ajax(window.location.pathname).get(this.search).send('json');
			this.size = ret.size;
			// 履歴を更新
			if (push_history && diff) window.history.pushState(null, '', `${location.pathname}?${this.search.toString()}`);

			// コールバック呼び出し
			if (this._callback) this._callback(ret.list);
			return ret.list;
		} catch (e) {
			if (this._error) this._error(e as Error);
			throw e;
		}
	}
}
