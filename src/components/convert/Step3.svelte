<script lang="ts">
	import { appData } from '$stores/appdata';
	import convState from '$states/converter.svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { IconFolder, IconFile, IconFolderPlus } from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onDestroy, onMount } from 'svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { handleKeyHint, keyHint } from '$states/keyhint.svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Input, Toggle } from 'waku/components';

	let name = $state(
		convState.name === ''
			? convState.source
				? (convState.source.split('/').pop() ?? '')
				: ''
			: (convState.name ?? '')
	);
	let targetLocation = $state(
		$appData.autoPop.enabled
			? ($appData.autoPop.converter.targetLocation ?? convState.target ?? '')
			: (convState.target ?? '')
	);
	let createFolder = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.createNewFolder ?? true) : true
	);

	let projectNamePlaceholder = $derived(($t`Enter project name` || '') as string);

	async function select() {
		targetLocation =
			(await open({
				directory: true,
				multiple: false,
			})) ?? '';
	}

	onMount(() => {
		return keyHint.register([['tab', $t`Navigate fields`]]);
	});

	// Update convState immediately when values change
	$effect(() => {
		convState.name = name;
		convState.target = targetLocation;
	});

	onDestroy(async () => {
		// Set Tauri AppState
		await wrapper(commands.convStateSet({ Name: name ?? '' }));
		await wrapper(commands.convStateSet({ CreateDirectory: createFolder }));
		await wrapper(commands.convStateSet({ Target: targetLocation }));
	});
</script>

<div class="flex w-full p-3 pb-3">
	<VStack gap="md" class="mx-auto w-full max-w-5xl">
		<BentoGrid cols={2} density="compact">
			<!-- Project Name -->
			<BentoItem>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconFile size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Project Name`}</span>
				</HStack>

				<VStack gap="sm">
					<Input
						id="project-name"
						bind:value={name}
						placeholder={projectNamePlaceholder}
						variant="seamless"
						onkeydown={(e) => {
							if (e.key === ' ') {
								e.preventDefault();
							}
						}}
					/>
					<p class="text-muted text-xs">
						{$t`This will be used as the base name for your output files`}
					</p>
				</VStack>
			</BentoItem>

			<!-- Create Folder Toggle -->
			<BentoItem>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconFolderPlus size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Create New Folder`}</span>
				</HStack>

				<button
					type="button"
					class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
					onclick={() => (createFolder = !createFolder)}
				>
					<VStack gap="xs" class="flex-1">
						<span class="text-sm font-medium">
							{createFolder ? $t`Create a new folder` : $t`Save directly in target location`}
						</span>
						<span class="text-muted text-xs">
							{#if createFolder}
								{$t`A folder named after the project will be created`}
							{:else}
								{$t`Files will be saved directly in target`}
							{/if}
						</span>
					</VStack>
					<div class="pointer-events-none ml-4">
						<Toggle bind:checked={createFolder} tabindex={-1} />
					</div>
				</button>
			</BentoItem>

			<!-- Target Location -->
			<BentoItem colspan={2}>
				<HStack gap="sm" align="center" class="text-muted mb-3">
					<IconFolder size={18} />
					<span class="text-xs font-bold tracking-wider uppercase">{$t`Output Location`}</span>
				</HStack>

				<button
					id="target-location"
					class="bg-surface-2 hover:bg-surface-1 hover:border-accent-500/50 group w-full rounded-lg border border-transparent p-3 text-left transition-all"
					use:handleKeyHint={{ keys: [['enter', $t`Select location`]] }}
					onclick={select}
				>
					<HStack gap="sm" align="center">
						<div class="text-accent-500">
							<IconFolder size={20} />
						</div>
						<VStack gap="xs" class="flex-1 overflow-hidden">
							<span class="text-sm font-medium">
								{#if !targetLocation}
									{$t`Select output folder`}
								{:else}
									{$t`Output folder selected`}
								{/if}
							</span>
							{#if targetLocation}
								<span class="text-muted truncate font-mono text-xs" title={targetLocation}>
									{truncatePath(targetLocation, 100)}
								</span>
							{/if}
						</VStack>
					</HStack>
				</button>
				<p class="text-muted mt-2 text-xs">
					{$t`Choose where the converted files will be saved`}
				</p>
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>
