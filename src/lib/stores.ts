import { writable } from "svelte/store"
import type { AppData, Toast } from "$lib/types"
import { appDataDefault, appDataKey } from "$lib/constants"
import { setTheme } from "$lib/functions"

// globally used stores
export const toast = writable<Toast | undefined>(undefined)

export const appData = writable<AppData>(
	JSON.parse(localStorage.getItem(appDataKey) || JSON.stringify({ ...appDataDefault })),
)
appData.subscribe(a => {
	localStorage.setItem(appDataKey, JSON.stringify(a))

	setTheme(a.theme)
})
