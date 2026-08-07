import adapter from '@sveltejs/adapter-static';
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf-8'));

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({ fallback: 'index.html' }),
		// Без этого version = Date.now(): каждая сборка меняет хеши всех чанков,
		// а web/build коммитится в репозиторий.
		version: { name: pkg.version }
	}
};

export default config;
