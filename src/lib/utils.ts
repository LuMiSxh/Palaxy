import { type AppData, FeatureFlag, SupportedLanguages, Theme } from '$types/appdata';
import { browser } from '$app/environment';
import type { Error, Result } from '$types';
import { addToast } from '$stores/toast';
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
 * Checks if a feature flag is enabled in the application data.
 *
 * @param {AppData} appData - The application data containing feature flags.
 * @param {FeatureFlag} flag - The feature flag to check.
 * @returns {boolean} - True if the feature flag is enabled, false otherwise.
 */
export function ffIsEnabled(appData: AppData, flag: FeatureFlag): boolean {
	return appData.featureFlags.includes(flag);
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

	addToast(
		`${output.error.type}: ${
			Object.hasOwn(output.error, 'data')
				? (
						output.error as {
							data: string;
						}
					).data
				: gt`An error occurred`
		}`,
		'error',
		3600
	);
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
