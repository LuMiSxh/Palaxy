// Shared types between the frontend and tauri
export enum FileFormat {
	PDF = "PDF" ,
	CBZ = "CBZ",
	EPUB = "EPUB"
}

export enum Direction {
	LTR = "Left to Right",
	RTL = "Right to Left"
}

export interface BaseResult {
	message: string | null
}

export interface AnalyzeResult extends BaseResult {
	chapter_per_volume: number[],
}

export interface ConvertResult extends BaseResult {}

// Frontend-only types
export enum InfoType {
	ERROR,
	SUCCESS,
	WARNING
}

export enum Theme {
	LIGHT = "Light",
	DARK = "Dark",
	SYSTEM = "System"
}

export interface Converter {
	running: boolean,
	sourceDirectory: string | null,
	targetDirectory: string | null,
	volumes: number[],
	direction: Direction,
	format: FileFormat,
	analysis: {
		running: boolean,
		sensibility: number,
	}
}

export interface AppData {
	theme: Theme,
	paths: {
		converted: string | null
	},
	popups: {
		help: boolean,
		info: boolean
	}
}

export interface Toast {
	type: InfoType,
	message: string,
	timeout: number
}
