export class MarkdownRenderCache<K, V> {
	private cache = new Map<K, V>();
	private maxSize: number;

	constructor(maxSize = 500) {
		// No. of entries in cache
		this.maxSize = maxSize;
	}

	get(key: K): V | undefined {
		return this.cache.get(key);
	}

	set(key: K, value: V): void {
		if (!this.cache.has(key) && this.cache.size >= this.maxSize) {
			const firstKey = this.cache.keys().next().value;
			if (firstKey !== undefined) {
				this.cache.delete(firstKey);
			}
		}
		this.cache.set(key, value);
	}

	has(key: K): boolean {
		return this.cache.has(key);
	}

	clear(): void {
		this.cache.clear();
	}
}
