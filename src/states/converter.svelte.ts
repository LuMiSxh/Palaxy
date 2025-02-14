import type { BundleFlag } from '$types';

class ConverterState {
	source: string | null = $state(null);
	bundle: BundleFlag | null = $state(null);
	bundleRecommendation: BundleFlag = $state('MANUAL');
	chapterSizes: number[] = $state([]);

	reset(): void {
		this.source = null;
		this.bundle = null;
		this.bundleRecommendation = 'MANUAL';
		this.chapterSizes = [];
	}
}

export default new ConverterState();

class ConverterStepState {
	index: number = $state(0);
	disablePrev: boolean = $state(true);
	disableNext: boolean = $state(true);

	reset(): void {
		this.index = 0;
		this.disablePrev = true;
		this.disableNext = true;
	}
}

export const stepState = new ConverterStepState();
