import type { BaseResult } from '$lib/types';

export enum BundlerFlag {
	NAME = 'NAME',
	IMAGE = 'IMAGE',
	MANUAL = 'MANUAL',
}

interface TabAnalyzeResult extends BaseResult {
	positive: Array<string>;
	negative: Array<string>;
	suggest: Array<string>;
	bundler: BundlerFlag;
}

interface TabVolumeResult extends BaseResult {
	totalChapters: number;
	totalVolumes: number | null;
	chaptersPerVolume: Array<number> | null;
}

export type { TabAnalyzeResult, TabVolumeResult };
