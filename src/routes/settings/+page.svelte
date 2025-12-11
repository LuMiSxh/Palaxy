<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { appData } from '$stores/appdata';
	import { Theme, SupportedLanguages, defaultAppData } from '$types/appdata';
	import { addToast } from '$states/toast.svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { openDialog } from '$states/dialog.svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { getName, getVersion, getTauriVersion } from '@tauri-apps/api/app';
	import { platform, arch, version } from '@tauri-apps/plugin-os';
	import { truncatePath } from '$lib/utils';
	import { slide } from 'svelte/transition';

	// Waku Imports
	import { VStack, Separator, BentoGrid, BentoItem, HStack } from 'waku/layout';
	import { Button, Toggle, Select, LoadingSpinner, Input, Label, Badge } from 'waku/components';
	import {
		IconBrush,
		IconKeyboard,
		IconAutomation,
		IconDeviceDesktop,
		IconRestore,
		IconFolder,
	} from '@tabler/icons-svelte';

	// System Info State
	let sysInfo = $state({
		appName: '',
		appVersion: '',
		tauriVersion: '',
		osName: '',
		osArch: '',
		osVersion: '',
		loading: true,
	});

	onMount(async () => {
		keyHint.register([
			['tab', $t`Navigate`],
			['shift+tab', $t`Navigate`],
			['enter', $t`Toggle`],
		]);
		try {
			const [name, ver, tauri, plat, architecture, osVer] = await Promise.all([
				getName(),
				getVersion(),
				getTauriVersion(),
				platform(),
				arch(),
				version(),
			]);
			sysInfo = {
				appName: name,
				appVersion: ver,
				tauriVersion: tauri,
				osName: plat,
				osArch: architecture,
				osVersion: osVer,
				loading: false,
			};
		} catch (e) {
			console.error(e);
			sysInfo.loading = false;
		}
	});

	async function selectTargetLocation() {
		const location = await open({ directory: true, multiple: false });
		if (location) $appData.autoPop.converter.targetLocation = location;
	}

	function handleReset() {
		openDialog({
			title: $t`Confirmation`,
			content: $t`Are you sure you want to reset the state of the application?`,
			onConfirm: () => {
				appData.set(defaultAppData);
				addToast($t`Reset successfully`, 'success');
			},
			onCancel: () => addToast($t`Reset canceled`, 'info'),
		});
	}

	const themeOptions = [
		{ value: Theme.System, label: $t`System` },
		{ value: Theme.Light, label: $t`Light` },
		{ value: Theme.Dark, label: $t`Dark` },
	];
	const langOptions = [
		{ value: SupportedLanguages.English, label: $t`English` },
		{ value: SupportedLanguages.German, label: $t`German` },
	];
	const formatOptions = [
		{ value: 'CBZ', label: 'CBZ' },
		{ value: 'EPUB', label: 'EPUB' },
	];
	const imageOptions = [
		{ value: 'None', label: $t`Original` },
		{ value: 'WebP', label: 'WebP' },
		{ value: 'AVIF', label: 'AVIF' },
	];
</script>

<div class="h-full w-full overflow-y-auto p-4">
	<VStack gap="lg" class="mx-auto max-w-6xl pb-20">
		<!-- Compact Header -->
		<div class="flex items-center justify-between">
			<h1 class="text-2xl font-bold">{$t`Settings`}</h1>
			<Button
				variant="ghost"
				onclick={handleReset}
				class="text-danger hover:bg-danger/10 h-8 px-3 text-sm"
			>
				<IconRestore size={16} />
				{$t`Reset`}
			</Button>
		</div>

		<BentoGrid cols={2} density="comfortable">
			<!-- 1. GENERAL (Theme/Lang) -->
			<BentoItem class="z-20 overflow-visible">
				<HStack align="center" gap="sm" class="text-muted mb-4">
					<IconBrush size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`General`}</span>
				</HStack>

				<VStack gap="md">
					<div class="flex flex-col gap-1">
						<Label text={$t`Theme`} class="mb-0!" />
						<Select options={themeOptions} bind:value={$appData.theme} variant="seamless" />
					</div>
					<Separator class="my-0!" />
					<div class="flex flex-col gap-1">
						<Label text={$t`Language`} class="mb-0!" />
						<Select options={langOptions} bind:value={$appData.language} variant="seamless" />
					</div>
				</VStack>
			</BentoItem>

			<!-- 2. INPUT (Toggles) -->
			<BentoItem>
				<HStack align="center" gap="sm" class="text-muted mb-4">
					<IconKeyboard size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Input`}</span>
				</HStack>

				<VStack gap="xs">
					<!-- Row 1: KeyHints -->
					<button
						class="hover:bg-surface-2 group -mx-2 flex w-full cursor-pointer items-center justify-between rounded-md p-2 text-left transition-colors"
						onclick={() => ($appData.showKeyHints = !$appData.showKeyHints)}
					>
						<div class="flex flex-col">
							<span class="group-hover:text-accent-500 text-sm font-medium transition-colors"
								>{$t`KeyHints`}</span
							>
							<span class="text-muted text-[10px]">Footer shortcuts</span>
						</div>
						<div class="pointer-events-none">
							<Toggle bind:checked={$appData.showKeyHints} tabindex={-1} />
						</div>
					</button>

					<Separator class="my-1!" />

					<!-- Row 2: Mouse Support -->
					<button
						class="hover:bg-surface-2 group -mx-2 flex w-full cursor-pointer items-center justify-between rounded-md p-2 text-left transition-colors"
						onclick={() => ($appData.mouseSupport = !$appData.mouseSupport)}
					>
						<div class="flex flex-col">
							<span class="group-hover:text-accent-500 text-sm font-medium transition-colors"
								>{$t`Mouse Support`}</span
							>
							<span class="text-muted text-[10px]">ActionHub button</span>
						</div>
						<div class="pointer-events-none">
							<Toggle bind:checked={$appData.mouseSupport} tabindex={-1} />
						</div>
					</button>
				</VStack>
			</BentoItem>

			<!-- 3. SYSTEM (Read Only) -->
			<BentoItem>
				<HStack align="center" gap="sm" class="text-muted mb-4">
					<IconDeviceDesktop size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`System`}</span>
				</HStack>

				{#if sysInfo.loading}
					<div class="flex h-full items-center justify-center"><LoadingSpinner size="sm" /></div>
				{:else}
					<div class="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
						<span class="text-muted">App</span>
						<span class="text-right font-mono">{sysInfo.appName} v{sysInfo.appVersion}</span>
						<span class="text-muted">Tauri</span>
						<span class="text-right font-mono">v{sysInfo.tauriVersion}</span>
						<Separator class="col-span-2 my-1!" />
						<span class="text-muted">OS</span>
						<span class="text-right font-mono">{sysInfo.osName}</span>
						<span class="text-muted">Kernel</span>
						<span class="text-right font-mono">{sysInfo.osVersion}</span>
					</div>
				{/if}
			</BentoItem>

			<!-- 4. AUTOMATION (Full Width Module) -->
			<BentoItem colspan={3} class="z-10 overflow-visible transition-all duration-300">
				<div class="mb-2 flex items-center justify-between">
					<HStack align="center" gap="sm" class="text-muted">
						<IconAutomation size={18} />
						<span class="text-xs font-bold tracking-wider uppercase">{$t`Automation`}</span>
					</HStack>

					<!-- Master Toggle inside Header -->
					<div class="flex items-center gap-3">
						<span
							class="text-sm font-medium {$appData.autoPop.enabled ? 'text-success' : 'text-muted'}"
						>
							{$appData.autoPop.enabled ? 'Enabled' : 'Disabled'}
						</span>
						<div class="pointer-events-auto">
							<!-- Ensure this toggle is clickable -->
							<Toggle variant="success" bind:checked={$appData.autoPop.enabled} />
						</div>
					</div>
				</div>

				{#if $appData.autoPop.enabled}
					<div
						transition:slide={{ duration: 300 }}
						class="border-waku-border/50 mt-4 border-t pt-4"
					>
						<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
							<!-- Col 1: Formats -->
							<VStack gap="md">
								<div class="flex flex-col gap-1">
									<Label text={$t`File Type`} class="mb-0!" />
									<Select
										options={formatOptions}
										bind:value={$appData.autoPop.converter.conversionType}
										variant="seamless"
									/>
								</div>
								<div class="flex flex-col gap-1">
									<Label text={$t`Image Format`} class="mb-0!" />
									<Select
										options={imageOptions}
										bind:value={$appData.autoPop.converter.imageFormat}
										variant="seamless"
									/>
								</div>
							</VStack>

							<!-- Col 2: Output Path -->
							<VStack gap="sm">
								<Label text={$t`Target Location`} class="mb-0!" />
								<button
									class="bg-surface-2 hover:bg-surface-0 hover:border-accent-500/50 group w-full rounded-lg border border-transparent p-3 text-left transition-all"
									onclick={selectTargetLocation}
								>
									<div class="text-accent-500 mb-1 flex items-center gap-2">
										<IconFolder size={16} />
										<span class="text-xs font-bold uppercase">Browse</span>
									</div>
									<div class="truncate font-mono text-sm opacity-70 group-hover:opacity-100">
										{$appData.autoPop.converter.targetLocation
											? truncatePath($appData.autoPop.converter.targetLocation)
											: $t`Select folder...`}
									</div>
								</button>

								<div class="mt-2">
									<Label text={$t`Volume Separator`} class="mb-1!" />
									<Input
										bind:value={$appData.autoPop.converter.volumeSeparator}
										variant="seamless"
										placeholder=" | "
									/>
								</div>
							</VStack>

							<!-- Col 3: Behaviors -->
							<VStack gap="xs">
								<Label text="Behavior" class="mb-1!" />

								<button
									class="hover:bg-surface-2 group -mx-2 flex w-full cursor-pointer items-center justify-between rounded-md p-2 text-left transition-colors"
									onclick={() =>
										($appData.autoPop.converter.createNewFolder =
											!$appData.autoPop.converter.createNewFolder)}
								>
									<div class="flex flex-col">
										<span class="group-hover:text-accent-500 text-sm font-medium transition-colors"
											>{$t`Create Folder`}</span
										>
										<span class="text-muted text-[10px]">New subfolder for output</span>
									</div>
									<div class="pointer-events-none">
										<Toggle
											bind:checked={$appData.autoPop.converter.createNewFolder}
											tabindex={-1}
										/>
									</div>
								</button>

								<button
									class="hover:bg-surface-2 group -mx-2 flex w-full cursor-pointer items-center justify-between rounded-md p-2 text-left transition-colors"
									onclick={() =>
										($appData.autoPop.converter.hideSingleVolumeNumber =
											!$appData.autoPop.converter.hideSingleVolumeNumber)}
								>
									<div class="flex flex-col">
										<span class="group-hover:text-accent-500 text-sm font-medium transition-colors"
											>{$t`Hide Numbers`}</span
										>
										<span class="text-muted text-[10px]">If only one volume exists</span>
									</div>
									<div class="pointer-events-none">
										<Toggle
											bind:checked={$appData.autoPop.converter.hideSingleVolumeNumber}
											tabindex={-1}
										/>
									</div>
								</button>
							</VStack>
						</div>
					</div>
				{:else}
					<div transition:slide={{ duration: 300 }} class="text-muted mt-2 text-sm">
						{$t`Enable to automatically apply these settings when selecting a new source folder.`}
					</div>
				{/if}
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>
