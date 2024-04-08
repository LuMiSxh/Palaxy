import { type AppData, type Converter, Direction, FileFormat, Theme } from '$lib/types';

export const appDataKey: string = 'db37c8';
export const appDataDefault: AppData = {
	theme: Theme.SYSTEM,
	paths: {
		converted: null
	},
	popups: {
		help: true,
		info: true
	}
}

export const converterDefault: Converter = {
	running: false,
	sourceDirectory: null,
	targetDirectory: null,
	volumes: [],
	direction: Direction.LTR,
	format: FileFormat.EPUB,
	analysis: {
		running: false,
		sensibility: 75
	}
}