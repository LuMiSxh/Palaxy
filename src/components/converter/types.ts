import type { BaseResult } from "$lib/types"

export enum BundleFlag {
	NAME = "NAME",
	IMAGE = "IMAGE",
	MANUAL = "MANUAL",
}

interface CommandAnalyze extends BaseResult {
	positive: Array<string>
	negative: Array<string>
	suggest: Array<string>
	flag: BundleFlag
}

interface CommandBundle extends BaseResult {
	total_chapters: number
	total_volumes: number | null
	chapter_sizes: Array<number> | null
}

interface CommandGetData extends BaseResult {
	data: Array<Array<string>>
}

export type { CommandAnalyze, CommandBundle, CommandGetData }
