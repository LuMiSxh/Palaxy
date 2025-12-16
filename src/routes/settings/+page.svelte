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
	import { VStack, BentoGrid, BentoItem, HStack } from 'waku/layout';
	import { Toggle, Select, LoadingSpinner, Input } from 'waku/components';
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
		const unregisterKeyHint = keyHint.register([
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

		return new Promise<void>((resolve) => {
			unregisterKeyHint();
			resolve();
		});
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

<div class="h-full w-full p-3">
	<VStack gap="md" class="mx-auto h-full max-w-6xl">
		<BentoGrid cols={3} density="comfortable" rows="auto auto 1fr">
			<!-- General Settings -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconBrush size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`General`}</span>
				</HStack>

				<VStack gap="sm">
					<div>
						<label for="theme" class="text-muted mb-2 block text-xs font-medium uppercase">
							{$t`Theme`}
						</label>
						<Select
							id="theme"
							options={themeOptions}
							bind:value={$appData.theme}
							style="seamless"
						/>
					</div>
					<div>
						<label for="language" class="text-muted mb-2 block text-xs font-medium uppercase">
							{$t`Language`}
						</label>
						<Select
							id="language"
							options={langOptions}
							bind:value={$appData.language}
							style="seamless"
						/>
					</div>
				</VStack>
			</BentoItem>

			<!-- Input Settings -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconKeyboard size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Input`}</span>
				</HStack>

				<VStack gap="sm">
					<button
						type="button"
						class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
						onclick={() => ($appData.showKeyHints = !$appData.showKeyHints)}
						onkeydown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault();
								$appData.showKeyHints = !$appData.showKeyHints;
							}
						}}
						data-keyhint={`enter;${$t`Toggle`}`}
					>
						<VStack gap="xs" class="flex-1">
							<span class="text-sm font-medium">{$t`KeyHints`}</span>
							<span class="text-muted text-xs">{$t`Show keyboard shortcuts in footer`}</span>
						</VStack>
						<div class="pointer-events-none">
							<Toggle bind:checked={$appData.showKeyHints} tabindex={-1} style="seamless" />
						</div>
					</button>

					<button
						type="button"
						class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
						onclick={() => ($appData.mouseSupport = !$appData.mouseSupport)}
						onkeydown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault();
								$appData.mouseSupport = !$appData.mouseSupport;
							}
						}}
						data-keyhint={`enter;${$t`Toggle`}`}
					>
						<VStack gap="xs" class="flex-1">
							<span class="text-sm font-medium">{$t`Mouse Support`}</span>
							<span class="text-muted text-xs">{$t`Show ActionHub button`}</span>
						</VStack>
						<div class="pointer-events-none">
							<Toggle bind:checked={$appData.mouseSupport} tabindex={-1} style="seamless" />
						</div>
					</button>
				</VStack>
			</BentoItem>

			<!-- System Info -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconDeviceDesktop size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`System`}</span>
				</HStack>

				{#if sysInfo.loading}
					<div class="flex h-full items-center justify-center"><LoadingSpinner size="sm" /></div>
				{:else}
					<VStack gap="sm">
						<div class="bg-surface-2 rounded-lg p-3">
							<VStack gap="xs">
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Application`}</span>
									<span class="font-mono">{sysInfo.appName} v{sysInfo.appVersion}</span>
								</HStack>
								<HStack justify="between" class="text-xs">
									<span class="text-muted">Tauri</span>
									<span class="font-mono">v{sysInfo.tauriVersion}</span>
								</HStack>
							</VStack>
						</div>
						<div class="bg-surface-2 rounded-lg p-3">
							<VStack gap="xs">
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Platform`}</span>
									<span class="font-mono">{sysInfo.osName}</span>
								</HStack>
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Architecture`}</span>
									<span class="font-mono">{sysInfo.osArch}</span>
								</HStack>
							</VStack>
						</div>
					</VStack>
				{/if}
			</BentoItem>

			<!-- Automation Section -->
			<BentoItem colspan={3} glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconAutomation size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Automation`}</span>
					<button
						type="button"
						class="ml-auto flex items-center gap-2"
						onclick={() => ($appData.autoPop.enabled = !$appData.autoPop.enabled)}
						onkeydown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault();
								$appData.autoPop.enabled = !$appData.autoPop.enabled;
							}
						}}
						data-keyhint={`enter;${$t`Toggle automation`}`}
					>
						<span
							class="text-sm font-medium {$appData.autoPop.enabled ? 'text-success' : 'text-muted'}"
						>
							{$appData.autoPop.enabled ? $t`Enabled` : $t`Disabled`}
						</span>
						<Toggle
							variant="success"
							bind:checked={$appData.autoPop.enabled}
							tabindex={-1}
							style="seamless"
						/>
					</button>
				</HStack>

				{#if $appData.autoPop.enabled}
					<div transition:slide={{ duration: 300 }}>
						<VStack gap="sm">
							<p class="text-muted text-xs">
								{$t`Automatically apply these settings when selecting a new source folder`}
							</p>

							<div class="grid grid-cols-3 gap-3">
								<!-- Conversion Settings -->
								<div>
									<label
										for="file-type"
										class="text-muted mb-2 block text-xs font-medium uppercase"
									>
										{$t`File Type`}
									</label>
									<Select
										id="file-type"
										options={formatOptions}
										bind:value={$appData.autoPop.converter.conversionType}
										style="seamless"
									/>
								</div>

								<div>
									<label
										for="image-format"
										class="text-muted mb-2 block text-xs font-medium uppercase"
									>
										{$t`Image Format`}
									</label>
									<Select
										id="image-format"
										options={imageOptions}
										bind:value={$appData.autoPop.converter.imageFormat}
										style="seamless"
									/>
								</div>

								<div>
									<label
										for="vol-separator"
										class="text-muted mb-2 block text-xs font-medium uppercase"
									>
										{$t`Volume Separator`}
									</label>
									<Input
										id="vol-separator"
										bind:value={$appData.autoPop.converter.volumeSeparator}
										style="seamless"
										placeholder=" | "
									/>
								</div>
							</div>

							<!-- Target Location -->
							<div>
								<label for="target-loc" class="text-muted mb-2 block text-xs font-medium uppercase">
									{$t`Target Location`}
								</label>
								<button
									type="button"
									id="target-loc"
									class="bg-surface-2 hover:bg-surface-1 group w-full cursor-pointer rounded-lg p-3 text-left transition-colors"
									onclick={selectTargetLocation}
								>
									<HStack gap="sm" align="center">
										<div
											class="bg-accent-500/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
										>
											<IconFolder size={20} class="text-accent-500" />
										</div>
										<VStack gap="xs" class="flex-1 overflow-hidden">
											{#if $appData.autoPop.converter.targetLocation}
												<span class="text-sm font-medium">{$t`Location selected`}</span>
												<span
													class="text-muted truncate font-mono text-xs"
													title={$appData.autoPop.converter.targetLocation}
												>
													{truncatePath($appData.autoPop.converter.targetLocation, 100)}
												</span>
											{:else}
												<span class="text-sm font-medium">{$t`No location selected`}</span>
												<span class="text-muted text-xs">{$t`Click to choose output folder`}</span>
											{/if}
										</VStack>
									</HStack>
								</button>
							</div>

							<!-- Behavior Toggles -->
							<div class="grid grid-cols-2 gap-3">
								<button
									type="button"
									class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
									onclick={() =>
										($appData.autoPop.converter.createNewFolder =
											!$appData.autoPop.converter.createNewFolder)}
									onkeydown={(e) => {
										if (e.key === 'Enter') {
											e.preventDefault();
											$appData.autoPop.converter.createNewFolder =
												!$appData.autoPop.converter.createNewFolder;
										}
									}}
									data-keyhint={`enter;${$t`Toggle`}`}
								>
									<VStack gap="xs" class="flex-1">
										<span class="text-sm font-medium">{$t`Create Folder`}</span>
										<span class="text-muted text-xs">{$t`Create new subfolder for output`}</span>
									</VStack>
									<div class="pointer-events-none">
										<Toggle
											bind:checked={$appData.autoPop.converter.createNewFolder}
											tabindex={-1}
											style="seamless"
										/>
									</div>
								</button>

								<button
									type="button"
									class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
									onclick={() =>
										($appData.autoPop.converter.hideSingleVolumeNumber =
											!$appData.autoPop.converter.hideSingleVolumeNumber)}
									onkeydown={(e) => {
										if (e.key === 'Enter') {
											e.preventDefault();
											$appData.autoPop.converter.hideSingleVolumeNumber =
												!$appData.autoPop.converter.hideSingleVolumeNumber;
										}
									}}
									data-keyhint={`enter;${$t`Toggle`}`}
								>
									<VStack gap="xs" class="flex-1">
										<span class="text-sm font-medium">{$t`Hide Volume Number`}</span>
										<span class="text-muted text-xs">{$t`When only one volume exists`}</span>
									</VStack>
									<div class="pointer-events-none">
										<Toggle
											bind:checked={$appData.autoPop.converter.hideSingleVolumeNumber}
											tabindex={-1}
											style="seamless"
										/>
									</div>
								</button>
							</div>
						</VStack>
					</div>
				{:else}
					<div transition:slide={{ duration: 300 }}>
						<p class="text-muted text-sm">
							{$t`Enable to automatically apply these settings when selecting a new source folder`}
						</p>
					</div>
				{/if}
			</BentoItem>

			<!-- Reset Button -->
			<BentoItem
				colspan={3}
				glass
				onclick={handleReset}
				data-keyhint={`enter;${$t`Reset settings`}`}
				class="cursor-pointer"
			>
				<HStack gap="sm" align="center" class="text-danger">
					<div
						class="bg-danger/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
					>
						<IconRestore size={20} class="text-danger" />
					</div>
					<VStack gap="xs" class="flex-1">
						<span class="text-sm font-semibold">{$t`Reset All Settings`}</span>
						<span class="text-muted text-xs"
							>{$t`Restore all settings to their default values`}</span
						>
					</VStack>
				</HStack>
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>
