import type { BundleFlag } from '$types';

class ConverterSvelte {
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

export default new ConverterSvelte();
