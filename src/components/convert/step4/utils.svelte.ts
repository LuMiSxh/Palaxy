import type { BundleResponse } from '$types';
import { addToast } from '$states/toast.svelte';
import { gt } from 'svelte-i18n-lingui';
import convState from '$states/converter.svelte';

class Step4State {
	/**
	 * The result of the last conversion operation.
	 */
	result: BundleResponse | null = $state(null);

	/**
	 * The sensibility value for the last conversion operation.
	 * Default is 75.
	 */
	sensibility: number = $state(75);

	/**
	 * Whether the conversion operation is in progress.
	 */
	loading: boolean = $state(false);
}

export const step4State = new Step4State();

export function checkChapterLimits(): void {
	if (step4State.result !== null && step4State.result.total_chapters) {
		const totalUsed = getTotalChapters();

		if (totalUsed > step4State.result.total_chapters) {
			addToast(
				gt({
					message:
						'The total number of chapters {usedNumber} used exceeds the detected number of chapters {detectedNumber}',
					values: {
						usedNumber: totalUsed,
						detectedNumber: step4State.result.total_chapters,
					},
				}),
				'warning'
			);
		}
	}
}

export function getTotalChapters() {
	return convState.chapterSizes.reduce((acc, cur) => acc + cur, 0);
}

export function getChaptersPercentage() {
	if (!step4State.result || !step4State.result.total_chapters) return 0;
	return Math.min(100, (getTotalChapters() / step4State.result.total_chapters) * 100);
}
