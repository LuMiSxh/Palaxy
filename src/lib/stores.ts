import { writable } from 'svelte/store';
import type { AppData, Converter, Toast } from '$lib/types';
import { appDataDefault, appDataKey, converterDefault } from '$lib/constants';
import { setTheme } from '$lib/functions';

// converter store
export const converter = writable<Converter>({...converterDefault})

// globally used stores
export const toast = writable<Toast | undefined>(undefined)

export const appData = writable<AppData>(
	JSON.parse(localStorage.getItem(appDataKey) || JSON.stringify({...appDataDefault}))
)
appData.subscribe(a => {
	localStorage.setItem(appDataKey, JSON.stringify(a))

	setTheme(a.theme)
	converter.update(c => {
		c.targetDirectory = a.paths.converted;
		return c;
	})
})

export const converterTab = writable<number>(0);
