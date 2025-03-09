import { jstsExtractor, svelteExtractor } from 'svelte-i18n-lingui/extractor';

export default {
	locales: ['en', 'de', "ja"],
	sourceLocale: 'en',
	catalogs: [
		{
			path: 'src/locales/{locale}',
			include: ['src']
		}
	],
	extractors: [jstsExtractor, svelteExtractor]
};
