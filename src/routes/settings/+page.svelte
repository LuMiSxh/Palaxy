<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { appData } from '$stores/appdata';
	import { Theme, SupportedLanguages, defaultAppData } from '$types/appdata';
	import { toast } from 'waku/components';
	import { keyHint } from '$states/keyhint.svelte';

	import { open } from '@tauri-apps/plugin-dialog';
	import { getName, getVersion, getTauriVersion } from '@tauri-apps/api/app';
	import { platform, arch, version } from '@tauri-apps/plugin-os';
	import { truncatePath } from '$lib/utils';
	import { slide } from 'svelte/transition';

	// Waku Imports
	import { VStack, BentoGrid, BentoItem, HStack } from 'waku/layout';
	import { Toggle, Select, LoadingSpinner, Input, Modal, Button } from 'waku/components';
	import {
		IconBrush,
		IconKeyboard,
		IconAutomation,
		IconDeviceDesktop,
		IconRestore,
		IconFolder,
		IconCheck,
		IconFileZip,
		IconPhoto,
		IconFolderPlus,
		IconSeparator,
	} from '@tabler/icons-svelte';

	// Modal State
	let showResetModal = $state(false);

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

	onMount(() => {
		const unregisterKeyHint = keyHint.register([['tab', $t`Navigate fields`]]);

		Promise.all([getName(), getVersion(), getTauriVersion(), platform(), arch(), version()])
			.then(([name, ver, tauri, plat, architecture, osVer]) => {
				sysInfo = {
					appName: name,
					appVersion: ver,
					tauriVersion: tauri,
					osName: plat,
					osArch: architecture,
					osVersion: osVer,
					loading: false,
				};
			})
			.catch((e) => {
				console.error(e);
				sysInfo.loading = false;
			});

		return () => {
			unregisterKeyHint();
		};
	});

	async function selectTargetLocation() {
		const location = await open({ directory: true, multiple: false });
		if (location) $appData.autoPop.converter.targetLocation = location;
	}

	function handleReset() {
		showResetModal = true;
	}

	function confirmReset() {
		appData.set(defaultAppData);
		toast({ title: $t`Settings reset successfully`, type: 'success' });
		showResetModal = false;
	}

	function cancelReset() {
		toast({ title: $t`Reset canceled`, type: 'info' });
		showResetModal = false;
	}

	let themeOptions = $derived([
		{ value: Theme.System, label: $t`System` },
		{ value: Theme.Light, label: $t`Light` },
		{ value: Theme.Dark, label: $t`Dark` },
	]);
	let langOptions = $derived([
		{ value: SupportedLanguages.English, label: $t`English` },
		{ value: SupportedLanguages.German, label: $t`German` },
	]);
	let formatOptions = $derived([
		{ value: 'CBZ', label: 'CBZ' },
		{ value: 'EPUB', label: 'EPUB' },
	]);
	let imageOptions = $derived([
		{ value: 'None', label: $t`Original (No Conversion)` },
		{ value: 'WebP', label: 'WebP' },
		{ value: 'AVIF', label: 'AVIF' },
	]);
</script>

<div class="h-full w-full overflow-y-auto p-3">
	<VStack gap="md" class="mx-auto h-full max-w-6xl">
		<!-- Header -->
		<VStack gap="xs">
			<h1 class="text-2xl font-bold">{$t`Settings`}</h1>
			<p class="text-muted text-sm">{$t`Customize your application preferences and defaults`}</p>
		</VStack>

		<BentoGrid cols={3} density="comfortable" rows="auto auto auto auto">
			<!-- Theme -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconBrush size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Theme`}</span>
				</HStack>

				<VStack gap="sm">
					<Select id="theme" options={themeOptions} bind:value={$appData.theme} style="seamless" />
					<p class="text-muted text-xs">
						{$t`Choose how the application looks. System follows your OS preference`}
					</p>
				</VStack>
			</BentoItem>

			<!-- Language -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconKeyboard size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Language`}</span>
				</HStack>

				<VStack gap="sm">
					<Select
						id="language"
						options={langOptions}
						bind:value={$appData.language}
						style="seamless"
					/>
					<p class="text-muted text-xs">
						{$t`Select your preferred language for the interface`}
					</p>
				</VStack>
			</BentoItem>

			<!-- System Info -->
			<BentoItem glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconDeviceDesktop size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`System Info`}</span>
				</HStack>

				{#if sysInfo.loading}
					<div class="flex h-full items-center justify-center">
						<LoadingSpinner size="sm" />
					</div>
				{:else}
					<VStack gap="sm">
						<div class="bg-surface-2 rounded-lg p-3">
							<VStack gap="xs">
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Application`}</span>
									<span class="font-mono font-medium">{sysInfo.appName} v{sysInfo.appVersion}</span>
								</HStack>
								<HStack justify="between" class="text-xs">
									<span class="text-muted">Tauri</span>
									<span class="font-mono font-medium">v{sysInfo.tauriVersion}</span>
								</HStack>
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Platform`}</span>
									<span class="font-mono font-medium">{sysInfo.osName}</span>
								</HStack>
								<HStack justify="between" class="text-xs">
									<span class="text-muted">{$t`Architecture`}</span>
									<span class="font-mono font-medium">{sysInfo.osArch}</span>
								</HStack>
							</VStack>
						</div>
					</VStack>
				{/if}
			</BentoItem>

			<!-- Keyboard Hints -->
			<BentoItem glass onclick={() => ($appData.showKeyHints = !$appData.showKeyHints)}>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconKeyboard size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Keyboard Hints`}</span>
				</HStack>

				<div
					class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 transition-colors"
				>
					<VStack gap="xs" class="flex-1">
						<span class="text-sm font-medium">
							{$appData.showKeyHints ? $t`Enabled` : $t`Disabled`}
						</span>
						<span class="text-muted text-xs">
							{$t`Show keyboard shortcuts in the footer`}
						</span>
					</VStack>
					<div class="pointer-events-none">
						<Toggle bind:checked={$appData.showKeyHints} tabindex={-1} style="seamless" />
					</div>
				</div>
			</BentoItem>

			<!-- Mouse Support -->
			<BentoItem glass onclick={() => ($appData.mouseSupport = !$appData.mouseSupport)}>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconDeviceDesktop size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Mouse Support`}</span>
				</HStack>

				<div
					class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 transition-colors"
				>
					<VStack gap="xs" class="flex-1">
						<span class="text-sm font-medium">
							{$appData.mouseSupport ? $t`Enabled` : $t`Disabled`}
						</span>
						<span class="text-muted text-xs">
							{$t`Show navigation buttons and action hub`}
						</span>
					</VStack>
					<div class="pointer-events-none">
						<Toggle bind:checked={$appData.mouseSupport} tabindex={-1} style="seamless" />
					</div>
				</div>
			</BentoItem>

			<!-- Placeholder for grid alignment -->
			<BentoItem glass class="pointer-events-none opacity-0">
				<div class="h-1"></div>
			</BentoItem>

			<!-- Automation Section -->
			<BentoItem colspan={3} glass>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconAutomation size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Automation Presets`}</span>
					<button
						type="button"
						class="hover:bg-surface-2 ml-auto flex items-center gap-2 rounded-lg p-2 transition-colors"
						onclick={() => ($appData.autoPop.enabled = !$appData.autoPop.enabled)}
					>
						<span
							class="text-sm font-medium transition-colors {$appData.autoPop.enabled
								? 'text-success'
								: 'text-muted'}"
						>
							{$appData.autoPop.enabled ? $t`Enabled` : $t`Disabled`}
						</span>
						<Toggle
							variant={$appData.autoPop.enabled ? 'success' : undefined}
							bind:checked={$appData.autoPop.enabled}
							tabindex={-1}
							style="seamless"
						/>
					</button>
				</HStack>

				{#if $appData.autoPop.enabled}
					<div transition:slide={{ duration: 300 }}>
						<VStack gap="md">
							<p class="text-muted text-xs">
								{$t`These settings will be automatically applied when starting a new conversion`}
							</p>

							<!-- File Format & Image Settings -->
							<VStack gap="sm">
								<span class="text-muted text-xs font-bold uppercase">{$t`Output Settings`}</span>
								<div class="grid grid-cols-3 gap-3">
									<div>
										<HStack gap="sm" align="center" class="text-muted mb-2">
											<IconFileZip size={16} />
											<label
												for="auto-file-type"
												class="text-xs font-medium tracking-wide uppercase"
											>
												{$t`Format`}
											</label>
										</HStack>
										<Select
											id="auto-file-type"
											options={formatOptions}
											bind:value={$appData.autoPop.converter.conversionType}
											style="seamless"
										/>
									</div>

									<div>
										<HStack gap="sm" align="center" class="text-muted mb-2">
											<IconPhoto size={16} />
											<label
												for="auto-image-format"
												class="text-xs font-medium tracking-wide uppercase"
											>
												{$t`Image Format`}
											</label>
										</HStack>
										<Select
											id="auto-image-format"
											options={imageOptions}
											bind:value={$appData.autoPop.converter.imageFormat}
											style="seamless"
										/>
									</div>

									<div>
										<HStack gap="sm" align="center" class="text-muted mb-2">
											<IconSeparator size={16} />
											<label for="auto-vol-sep" class="text-xs font-medium tracking-wide uppercase">
												{$t`Volume Separator`}
											</label>
										</HStack>
										<Input
											id="auto-vol-sep"
											bind:value={$appData.autoPop.converter.volumeSeparator}
											style="seamless"
											placeholder=" | "
											onkeydown={(e) => {
												if (e.key === ' ') {
													e.stopPropagation();
												}
											}}
										/>
									</div>
								</div>
							</VStack>

							<!-- Target Location -->
							<VStack gap="sm">
								<HStack gap="sm" align="center" class="text-muted">
									<IconFolder size={16} />
									<span class="text-xs font-bold uppercase">{$t`Target Location`}</span>
									{#if $appData.autoPop.converter.targetLocation}
										<div
											class="bg-success/20 ml-auto flex h-5 w-5 items-center justify-center rounded-full"
										>
											<IconCheck size={12} class="text-success" />
										</div>
									{/if}
								</HStack>

								<button
									type="button"
									class="bg-surface-2 hover:bg-surface-1 group w-full cursor-pointer rounded-lg p-3 text-left transition-colors"
									onclick={selectTargetLocation}
								>
									<HStack gap="sm" align="center">
										<div
											class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {$appData
												.autoPop.converter.targetLocation
												? 'bg-accent-500/20'
												: 'bg-surface-1'}"
										>
											<IconFolder
												size={20}
												class={$appData.autoPop.converter.targetLocation
													? 'text-accent-500'
													: 'text-muted'}
											/>
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
												<span class="text-muted text-xs"
													>{$t`Click to choose default output folder`}</span
												>
											{/if}
										</VStack>
									</HStack>
								</button>
							</VStack>

							<!-- Behavior Toggles -->
							<VStack gap="sm">
								<span class="text-muted text-xs font-bold uppercase">{$t`Behavior`}</span>
								<div class="grid grid-cols-2 gap-3">
									<button
										type="button"
										class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
										onclick={() =>
											($appData.autoPop.converter.createNewFolder =
												!$appData.autoPop.converter.createNewFolder)}
									>
										<HStack gap="sm" align="center" class="flex-1">
											<div
												class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full {$appData
													.autoPop.converter.createNewFolder
													? 'bg-accent-500/20'
													: 'bg-surface-1'}"
											>
												<IconFolderPlus
													size={16}
													class={$appData.autoPop.converter.createNewFolder
														? 'text-accent-500'
														: 'text-muted'}
												/>
											</div>
											<VStack gap="xs" class="flex-1">
												<span class="text-sm font-medium">{$t`Create Project Folder`}</span>
												<span class="text-muted text-xs">{$t`New folder for each project`}</span>
											</VStack>
										</HStack>
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
									>
										<HStack gap="sm" align="center" class="flex-1">
											<div
												class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full {$appData
													.autoPop.converter.hideSingleVolumeNumber
													? 'bg-accent-500/20'
													: 'bg-surface-1'}"
											>
												<IconFileZip
													size={16}
													class={$appData.autoPop.converter.hideSingleVolumeNumber
														? 'text-accent-500'
														: 'text-muted'}
												/>
											</div>
											<VStack gap="xs" class="flex-1">
												<span class="text-sm font-medium">{$t`Hide Volume Number`}</span>
												<span class="text-muted text-xs">{$t`When only one volume exists`}</span>
											</VStack>
										</HStack>
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
						</VStack>
					</div>
				{:else}
					<div transition:slide={{ duration: 300 }}>
						<div class="bg-surface-2 rounded-lg p-6 text-center">
							<VStack gap="sm" align="center">
								<div class="bg-surface-1 flex h-12 w-12 items-center justify-center rounded-full">
									<IconAutomation size={24} class="text-muted" />
								</div>
								<p class="text-muted text-sm">
									{$t`Enable automation to pre-fill converter settings with your preferred defaults`}
								</p>
							</VStack>
						</div>
					</div>
				{/if}
			</BentoItem>

			<!-- Reset Button -->
			<BentoItem
				colspan={3}
				glass
				onclick={handleReset}
				data-keyhint={`enter;${$t`Reset settings`}`}
				class="mb-3"
			>
				<HStack gap="sm" align="center">
					<div
						class="bg-danger/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
					>
						<IconRestore size={20} class="text-danger" />
					</div>
					<VStack gap="xs" class="flex-1">
						<span class="text-danger text-sm font-semibold">{$t`Reset All Settings`}</span>
						<span class="text-muted text-xs">
							{$t`Restore all settings to their default values`}
						</span>
					</VStack>
				</HStack>
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>

<Modal bind:open={showResetModal} size="sm">
	<VStack gap="md" class="p-6">
		<VStack gap="xs">
			<h2 class="text-lg font-semibold">{$t`Confirmation`}</h2>
			<p class="text-muted text-sm">
				{$t`Are you sure you want to reset all settings to their default values?`}
			</p>
		</VStack>
		<HStack gap="sm" justify="end">
			<Button variant="neutral" style="subtle" onclick={cancelReset}>
				{$t`Cancel`}
			</Button>
			<Button variant="danger" onclick={confirmReset}>
				{$t`Reset`}
			</Button>
		</HStack>
	</VStack>
</Modal>
