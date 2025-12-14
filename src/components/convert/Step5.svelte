<script lang="ts">
	import { appData } from '$stores/appdata';
	import convState from '$states/converter.svelte';
	import { commands, type Direction, type FileFormat, type ImageOutputFormat } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { onDestroy, onMount } from 'svelte';
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

	const formatOptions = [
		{ value: 'CBZ', label: 'CBZ' },
		{ value: 'EPUB', label: 'EPUB' },
	];

	const directionOptions = [
		{ value: 'Left to Right', label: $t`Left to Right` },
		{ value: 'Right to Left', label: $t`Right to Left` },
	];

	const imageFormatOptions = [
		{ value: 'None', label: $t`Original (No Conversion)` },
		{ value: 'WebP', label: 'WebP' },
		{ value: 'AVIF', label: 'AVIF' },
	];

	onMount(() => {
		return keyHint.register([['tab', $t`Navigate fields`]]);
	});

	onDestroy(async () => {
		// Set Tauri AppState
		await wrapper(commands.convStateSet({ Direction: readingDirection }));
		await wrapper(commands.convStateSet({ Format: fileFormat }));
		await wrapper(commands.convStateSet({ ImageFormat: imageFormat }));
		await wrapper(commands.convStateSet({ HideSingleVolumeNumber: hideSingleVolumeNumber }));
		await wrapper(commands.convStateSet({ VolumeSeparator: volumeSeparator }));
	});
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={2} density="comfortable" class="h-full">
		<!-- File Format -->
		<BentoItem variant="glass">
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFileText size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`File Format`}</span>
			</HStack>

			<VStack gap="sm">
				<Select id="file-type" options={formatOptions} bind:value={fileFormat} variant="seamless" />
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
		<BentoItem variant="glass">
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconDirection size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Reading Direction`}</span>
				{#if fileFormat !== 'EPUB'}
					<Badge variant="secondary" class="ml-auto text-xs">{$t`EPUB only`}</Badge>
				{/if}
			</HStack>

			<VStack gap="sm">
				<Select
					id="reading-direction"
					options={directionOptions}
					bind:value={readingDirection}
					disabled={fileFormat !== 'EPUB'}
					variant="seamless"
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
		<BentoItem variant="glass">
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconPhoto size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Image Format`}</span>
			</HStack>

			<VStack gap="sm">
				<Select
					id="image-format"
					options={imageFormatOptions}
					bind:value={imageFormat}
					variant="seamless"
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
		<BentoItem variant="glass" onclick={() => (hideSingleVolumeNumber = !hideSingleVolumeNumber)}>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFileText size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Single Volume Number`}</span>
			</HStack>

			<div
				class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
			>
				<VStack gap="xs" class="flex-1">
					<span class="text-sm font-medium">
						{hideSingleVolumeNumber ? $t`Number will be hidden` : $t`Number will be shown`}
					</span>
					<span class="text-muted text-xs">
						{#if hideSingleVolumeNumber}
							{$t`Volume number won't be appended for single volumes`}
						{:else}
							{$t`Volume number will always be shown`}
						{/if}
					</span>
				</VStack>
				<div class="pointer-events-none ml-4">
					<Toggle bind:checked={hideSingleVolumeNumber} tabindex={-1} />
				</div>
			</div>
		</BentoItem>

		<!-- Volume Separator -->
		<BentoItem colspan={2} variant="glass">
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconSeparator size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Volume Separator`}</span>
			</HStack>

			<VStack gap="sm">
				<Input
					id="volume-separator"
					bind:value={volumeSeparator}
					placeholder=" | "
					variant="seamless"
				/>
				<p class="text-muted text-xs">
					{$t`Separator between volume name and volume number (e.g., "My Manga | 1")`}
				</p>
			</VStack>
		</BentoItem>

		<!-- Preview Section -->
		<BentoItem colspan={2} variant="surface">
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<div class="bg-success/20 h-1.5 w-1.5 rounded-full"></div>
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Output Preview`}</span>
			</HStack>

			<div class="grid grid-cols-2 gap-4">
				<HStack justify="between" align="center">
					<span class="text-muted text-sm">{$t`File Format`}</span>
					<span class="text-sm font-medium">{fileFormat}</span>
				</HStack>
				{#if fileFormat === 'EPUB'}
					<HStack justify="between" align="center">
						<span class="text-muted text-sm">{$t`Reading Direction`}</span>
						<span class="text-sm">{readingDirection}</span>
					</HStack>
				{/if}
				<HStack justify="between" align="center">
					<span class="text-muted text-sm">{$t`Image Format`}</span>
					<span class="text-sm">{imageFormat}</span>
				</HStack>
				<HStack justify="between" align="center" class="col-span-2">
					<span class="text-muted text-sm">{$t`Example Filename`}</span>
					<span class="truncate font-mono text-xs">
						{convState.name || 'Project'}{volumeSeparator}1.{fileFormat.toLowerCase()}
					</span>
				</HStack>
			</div>
		</BentoItem>
	</BentoGrid>
</div>
