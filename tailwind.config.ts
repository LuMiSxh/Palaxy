import containerQueries from '@tailwindcss/container-queries';
import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';
import type { Config } from 'tailwindcss';

import { contentPath, skeleton } from '@skeletonlabs/skeleton/plugin';
import { nosh } from '@skeletonlabs/skeleton/themes';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}', contentPath(import.meta.url, 'svelte')],
	darkMode: 'class',
	theme: {
		extend: {}
	},

	plugins: [
		skeleton({
			themes: [nosh]
		}),
		typography,
		forms,
		containerQueries
	]
} satisfies Config;
