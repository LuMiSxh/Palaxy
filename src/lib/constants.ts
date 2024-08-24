import { type AppData, Theme } from "$lib/types"

export const appDataKey: string = "db37c8"
export const appDataDefault: AppData = {
	theme: Theme.SYSTEM,
	paths: {
		converted: null,
	},
	popups: {
		help: true,
		info: true,
	},
}
