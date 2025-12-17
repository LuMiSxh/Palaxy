import { SupportedLanguages, Theme } from '$types/appdata';
import { browser } from '$app/environment';
import type { Error, Result } from '$types';
import { toast } from 'waku/components';
import { gt, locale } from 'svelte-i18n-lingui';

const LIGHT = 'alya';
const DARK = 'alya-dark';

/**
 * Sets the theme of the application.
 *
 * @param {Theme} theme - The theme to set (LIGHT, DARK, or SYSTEM).
 */
export function setTheme(theme: Theme) {
	if (!browser) return;

	switch (theme) {
		case Theme.Light:
			document.documentElement.classList.remove('dark');
			document.documentElement.setAttribute('data-theme', LIGHT);
			break;
		case Theme.Dark:
			document.documentElement.classList.add('dark');
			document.documentElement.setAttribute('data-theme', DARK);
			break;
		case Theme.System:
			if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
				document.documentElement.classList.add('dark');
				document.documentElement.setAttribute('data-theme', DARK);
			} else {
				document.documentElement.classList.remove('dark');
				document.documentElement.setAttribute('data-theme', LIGHT);
			}
			break;
	}
}

/**
 * Converts a string to a title case.
 *
 * @param {string} str - The string to convert.
 * @returns {string} - The converted string in title case.
 */
export function convertToTitleCase(str: string): string {
	if (!str) {
		return '';
	}
	return str.toLowerCase().replace(/\b\w/g, (s) => s.toUpperCase());
}

/**
 * Wraps a promise and returns the result if successful, or null if an error occurs.
 *
 * @template T
 * @param {Promise<Result<T, Error>>} input - The promise to wrap.
 * @returns {Promise<T | null>} - The result of the promise if successful, or null if an error occurs.
 */
export async function wrapper<T>(input: Promise<Result<T, Error>>): Promise<T | null> {
	const output = await input;

	if (output.status === 'ok') return output.data;

	let message = gt`An error occurred`;

	// noinspection SuspiciousTypeOfGuard
	if (typeof output.error === 'string') {
		message = output.error;
	} else if (Object.hasOwn(output.error, 'data')) {
		message = (output.error as { data: string }).data;
	}

	toast({
		title: `${output.error.type ? output.error.type + ': ' : ''}${message}`,
		type: 'error',
		timeout: 3600,
	});
	return null;
}

/**
 * Sets the locale for the application.
 *
 * This function dynamically imports the locale messages for the specified language
 * and sets the locale using the `svelte-i18n-lingui` library.
 *
 * @param {SupportedLanguages} lang - The language to set the locale to.
 * @returns {Promise<void>} - A promise that resolves when the locale is set.
 */
export async function setLocale(lang: SupportedLanguages): Promise<void> {
	const { messages } = await import(`../locales/${lang}.ts`);
	locale.set(lang, messages);
}

/**
 * Truncates a file path to a specified maximum length.
 *
 * @param {string} path - The file path to truncate.
 * @param {number} maxLength - The maximum length of the truncated path (default is 50).
 * @returns {string} - The truncated file path.
 */
export function truncatePath(path: string, maxLength: number = 50): string {
	if (!path || path.length <= maxLength) return path;

	const parts = path.split('/');
	const fileName = parts.pop() || '';
	const dirName = parts[0] || '';

	// Preserve first directory and filename, truncate the middle
	if (fileName.length + dirName.length + 5 >= maxLength) {
		// If filename itself is very long
		return fileName.length > maxLength - 5
			? fileName.substring(0, maxLength - 5) + '...'
			: fileName;
	}

	return dirName + '/.../' + fileName;
}
