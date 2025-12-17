<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { msg, t } from 'svelte-i18n-lingui';
	import convState from '$states/converter.svelte';
	import { commands, type ConvState } from '$types';
	import { truncatePath, wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import {
		IconBook,
		IconPhoto,
		IconVocabulary,
		IconFolder,
		IconDirection,
		IconSettings,
	} from '@tabler/icons-svelte';

	// Waku Imports
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Badge } from 'waku/components';

	let convStateData: ConvState | null = $state(null);
	let unregisterKeyHint: () => void;
	let totalImages = $state(0);
	let coverImages = $state(0);
	let images: string[][] = $state([]);
	let scrollContainer: HTMLDivElement;

	// eslint-disable-next-line @typescript-eslint/no-unused-expressions
	msg`Left to Right`;
	// eslint-disable-next-line @typescript-eslint/no-unused-expressions
	msg`Right to Left`;

	// Function to calculate the total number of images in a volume
	function getVolumeImageCount(volumeIndex: number): number {
		if (!convStateData?.volume_sizes || !convStateData?.data) return 0;

		let chapterIndex = 0;
		for (let i = 0; i < volumeIndex; i++) {
			chapterIndex += convStateData.volume_sizes[i];
		}

		let count = 0;
		for (let i = 0; i < convStateData.volume_sizes[volumeIndex]; i++) {
			if (chapterIndex + i < images.length) {
				count += images[chapterIndex + i].length;
			}
		}

		return count;
	}

	// Calculate max volume image count for chart scaling
	function calculateMaxVolumeImageCount(): void {
		if (!convStateData?.volume_sizes) return;

		let max = 0;
		for (let i = 0; i < convStateData.volume_sizes.length; i++) {
			const count = getVolumeImageCount(i);
			if (count > max) max = count;
		}
	}

	// Calculate total images and cover images
	function calculateImageStats() {
		if (!convStateData?.data) return;

		totalImages = images.reduce((sum: number, chapter: string[]) => sum + chapter.length, 0);
		coverImages = convStateData?.volume_sizes?.length || 0;
		calculateMaxVolumeImageCount();
	}

	onMount(async () => {
		unregisterKeyHint = keyHint.register([
			['arrowup', $t`Scroll up`],
			['arrowdown', $t`Scroll down`],
		]);

		const result = await wrapper(commands.convStateGet());
		if (result !== null && result.payload !== null) {
			convStateData = result.payload;
			images = convStateData.edited_data || convStateData.data;
			calculateImageStats();
		}

		if (scrollContainer) {
			scrollContainer.focus();
		}
	});

	onDestroy(() => {
		if (unregisterKeyHint) unregisterKeyHint();
	});
</script>

<div class="h-full w-full p-3">
	<div class="h-full overflow-y-auto" bind:this={scrollContainer} tabindex="0" role="tab">
		<VStack gap="md" class="mx-auto max-w-6xl pb-4">
			<BentoGrid cols={3} density="compact">
				<!-- Statistics Cards -->
				<BentoItem glass>
					<div class="flex flex-col items-center justify-center p-2 text-center">
						<IconBook class="text-accent-500 mb-3" size={32} />
						<div class="text-3xl font-bold">{convStateData?.volume_sizes?.length ?? 0}</div>
						<div class="text-muted mt-1 text-sm">{$t`Volumes`}</div>
					</div>
				</BentoItem>

				<BentoItem glass>
					<div class="flex flex-col items-center justify-center p-2 text-center">
						<IconVocabulary class="text-accent-500 mb-3" size={32} />
						<div class="text-3xl font-bold">{convStateData?.data?.length ?? 0}</div>
						<div class="text-muted mt-1 text-sm">{$t`Chapters`}</div>
					</div>
				</BentoItem>

				<BentoItem glass>
					<div class="flex flex-col items-center justify-center p-2 text-center">
						<IconPhoto class="text-accent-500 mb-3" size={32} />
						<div class="text-3xl font-bold">{totalImages - (convState.excludedImages || 0)}</div>
						<div class="text-muted mt-1 text-sm">{$t`Images`}</div>
					</div>
				</BentoItem>

				<!-- Project Information -->
				<BentoItem colspan={2} glass>
					<HStack align="center" gap="sm" class="text-muted mb-3">
						<IconFolder size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Project`}</span>
					</HStack>

					<VStack gap="sm">
						<div>
							<div class="text-muted mb-1 text-xs">{$t`Name`}</div>
							<div class="bg-surface-2 rounded-lg px-3 py-2 font-medium">
								{convStateData?.name || $t`Unnamed Project`}
							</div>
						</div>

						<div>
							<div class="text-muted mb-1 text-xs">{$t`Source`}</div>
							<div
								class="bg-surface-2 truncate rounded-lg px-3 py-2 font-mono text-sm"
								title={convStateData?.source || ''}
							>
								{truncatePath(convStateData?.source || $t`Not specified`)}
							</div>
						</div>

						<div>
							<div class="text-muted mb-1 text-xs">{$t`Target`}</div>
							<div
								class="bg-surface-2 truncate rounded-lg px-3 py-2 font-mono text-sm"
								title={convState.target || ''}
							>
								{truncatePath(convState.target || $t`Not specified`)}
							</div>
						</div>
					</VStack>
				</BentoItem>

				<!-- Output Settings -->
				<BentoItem glass>
					<HStack align="center" gap="sm" class="text-muted mb-3">
						<IconSettings size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Settings`}</span>
					</HStack>

					<VStack gap="sm">
						<div>
							<div class="text-muted mb-1 text-xs">{$t`Format`}</div>
							<Badge variant="primary" class="w-full justify-center">
								{convStateData?.format || $t`Not specified`}
							</Badge>
						</div>

						<div>
							<div class="text-muted mb-1 text-xs">{$t`Direction`}</div>
							<HStack align="center" gap="xs">
								<IconDirection size={14} class="text-muted" />
								<span class="text-sm">
									{convStateData ? $t(convStateData.direction) : $t`Left to Right`}
								</span>
							</HStack>
						</div>

						<div>
							<div class="text-muted mb-1 text-xs">{$t`Image Format`}</div>
							<div class="text-sm">{convStateData?.image_format ?? $t`None`}</div>
						</div>

						<div>
							<div class="text-muted mb-1 text-xs">{$t`New Folder`}</div>
							<HStack align="center" gap="xs">
								<div
									class="h-4 w-7 rounded-full transition-colors {convStateData?.create_directory
										? 'bg-accent-500'
										: 'bg-surface-0'}"
								>
									<div
										class="h-4 w-4 rounded-full bg-white shadow-sm transition-transform {convStateData?.create_directory
											? 'translate-x-3'
											: 'translate-x-0'}"
									></div>
								</div>
								<span class="text-sm">
									{convStateData?.create_directory ? $t`Yes` : $t`No`}
								</span>
							</HStack>
						</div>
					</VStack>
				</BentoItem>

				<!-- Image Statistics -->
				<BentoItem colspan={3} glass>
					<HStack align="center" gap="sm" class="text-muted mb-3">
						<IconPhoto size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Image Statistics`}</span>
					</HStack>

					<div class="grid grid-cols-4 gap-3">
						<div class="bg-surface-2 rounded-lg p-3 text-center">
							<div class="text-muted mb-1 text-xs">{$t`Total`}</div>
							<div class="text-xl font-semibold">{totalImages}</div>
						</div>
						<div class="bg-surface-2 rounded-lg p-3 text-center">
							<div class="text-muted mb-1 text-xs">{$t`Covers`}</div>
							<div class="text-xl font-semibold">{coverImages}</div>
						</div>
						<div class="bg-surface-2 rounded-lg p-3 text-center">
							<div class="text-muted mb-1 text-xs">{$t`Excluded`}</div>
							<div class="text-xl font-semibold">{convState.excludedImages || 0}</div>
						</div>
						<div class="bg-surface-2 rounded-lg p-3 text-center">
							<div class="text-muted mb-1 text-xs">{$t`Added`}</div>
							<div class="text-xl font-semibold">{convState.newImages || 0}</div>
						</div>
					</div>
				</BentoItem>
			</BentoGrid>
		</VStack>
	</div>
</div>

<style>
	/* Remove focus indicators while preserving accessibility */
	div[tabindex='0']:focus {
		outline: none;
		box-shadow: none;
		border-color: transparent;
	}
</style>
