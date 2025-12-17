<script lang="ts">
	import { appData } from '$stores/appdata';
	import { commands, type Direction, type FileFormat, type ImageOutputFormat } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Input, Select, Toggle, Badge } from 'waku/components';
	import { IconFileText, IconPhoto, IconSeparator, IconDirection } from '@tabler/icons-svelte';

	let fileFormat: FileFormat = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.conversionType ?? 'CBZ') : 'CBZ'
	);
	let readingDirection: Direction = $state('Left to Right');
	let imageFormat: ImageOutputFormat = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.imageFormat ?? 'WebP') : 'WebP'
	);
	let hideSingleVolumeNumber = $state(
		$appData.autoPop.enabled ? $appData.autoPop.converter.hideSingleVolumeNumber : false
	);
	let volumeSeparator = $state(
		$appData.autoPop.enabled ? $appData.autoPop.converter.volumeSeparator : ' | '
	);

	let formatOptions = $derived([
		{ value: 'CBZ', label: 'CBZ' },
		{ value: 'EPUB', label: 'EPUB' },
	]);

	let directionOptions = $derived([
		{ value: 'Left to Right', label: $t`Left to Right` },
		{ value: 'Right to Left', label: $t`Right to Left` },
	]);

	let imageFormatOptions = $derived([
		{ value: 'None', label: $t`Original (No Conversion)` },
		{ value: 'WebP', label: 'WebP' },
		{ value: 'AVIF', label: 'AVIF' },
	]);

	onMount(() => {
		return keyHint.register([['tab', $t`Navigate fields`]]);
	});

	// Save state immediately when values change instead of in onDestroy
	$effect(() => {
		if (fileFormat) {
			wrapper(commands.convStateSet({ Format: fileFormat }));
		}
	});

	$effect(() => {
		if (readingDirection) {
			wrapper(commands.convStateSet({ Direction: readingDirection }));
		}
	});

	$effect(() => {
		if (imageFormat) {
			wrapper(commands.convStateSet({ ImageFormat: imageFormat }));
		}
	});

	$effect(() => {
		wrapper(commands.convStateSet({ HideSingleVolumeNumber: hideSingleVolumeNumber }));
	});

	$effect(() => {
		if (volumeSeparator !== undefined) {
			wrapper(commands.convStateSet({ VolumeSeparator: volumeSeparator }));
		}
	});
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={2} density="comfortable" rows="auto auto auto 1.5fr" class="h-full">
		<!-- File Format -->
		<BentoItem glass>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFileText size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`File Format`}</span>
			</HStack>

			<VStack gap="sm">
				<Select id="file-type" options={formatOptions} bind:value={fileFormat} style="seamless" />
				<p class="text-muted text-xs">
					{#if fileFormat === 'CBZ'}
						{$t`Comic Book Archive format - widely supported by comic readers`}
					{:else if fileFormat === 'EPUB'}
						{$t`eBook format with better metadata support and reading options`}
					{/if}
				</p>
			</VStack>
		</BentoItem>

		<!-- Reading Direction -->
		<BentoItem glass>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconDirection size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Reading Direction`}</span>
				{#if fileFormat !== 'EPUB'}
					<Badge variant="neutral" class="ml-auto text-xs">{$t`EPUB only`}</Badge>
				{/if}
			</HStack>

			<VStack gap="sm">
				<Select
					id="reading-direction"
					options={directionOptions}
					bind:value={readingDirection}
					disabled={fileFormat !== 'EPUB'}
					style="seamless"
				/>
				<p class="text-muted text-xs">
					{#if fileFormat !== 'EPUB'}
						{$t`Reading direction is only applicable to EPUB format`}
					{:else}
						{$t`Choose the reading direction for your manga (typically Right to Left for Japanese manga)`}
					{/if}
				</p>
			</VStack>
		</BentoItem>

		<!-- Image Format -->
		<BentoItem glass>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconPhoto size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Image Format`}</span>
			</HStack>

			<VStack gap="sm">
				<Select
					id="image-format"
					options={imageFormatOptions}
					bind:value={imageFormat}
					style="seamless"
				/>
				<p class="text-muted text-xs">
					{#if imageFormat === 'None'}
						{$t`Images will keep their original format (fastest, larger file size)`}
					{:else if imageFormat === 'WebP'}
						{$t`Images will be converted to WebP (good compression, widely supported)`}
					{:else if imageFormat === 'AVIF'}
						{$t`Images will be converted to AVIF (best compression, slower conversion)`}
					{/if}
				</p>
			</VStack>
		</BentoItem>

		<!-- Hide Single Volume Number Toggle -->
		<BentoItem glass onclick={() => (hideSingleVolumeNumber = !hideSingleVolumeNumber)}>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFileText size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Volume Numbering`}</span>
			</HStack>

			<div
				class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
			>
				<VStack gap="xs" class="flex-1">
					<span class="text-sm font-medium">
						{hideSingleVolumeNumber ? $t`Hide for single volume` : $t`Always show number`}
					</span>
					<span class="text-muted text-xs">
						{#if hideSingleVolumeNumber}
							{$t`Volume number won't be added when there's only one volume`}
						{:else}
							{$t`Volume number will be included regardless of volume count`}
						{/if}
					</span>
				</VStack>
				<div class="pointer-events-none ml-4">
					<Toggle bind:checked={hideSingleVolumeNumber} tabindex={-1} style="seamless" />
				</div>
			</div>
		</BentoItem>

		<!-- Volume Separator -->
		<BentoItem colspan={2} glass>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconSeparator size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Volume Separator`}</span>
			</HStack>

			<VStack gap="sm">
				<Input
					id="volume-separator"
					bind:value={volumeSeparator}
					placeholder=" | "
					style="seamless"
					onkeydown={(e) => {
						if (e.key === ' ') {
							e.stopPropagation();
						}
					}}
				/>
				<p class="text-muted text-xs">
					{$t`Separator between volume name and volume number (e.g., "My Manga | 1")`}
				</p>
			</VStack>
		</BentoItem>
	</BentoGrid>
</div>
