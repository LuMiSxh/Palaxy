<script lang="ts">
	import { appData } from '$stores/appdata';
	import convState from '$states/converter.svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { IconFolder, IconFile, IconFolderPlus, IconCheck } from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Input, Toggle } from 'waku/components';

	let name = $state(
		convState.name === ''
			? convState.source
				? (convState.source
						.split('/')
						.pop()
						?.replace(/\.[^/.]+$/, '') ?? '')
				: ''
			: (convState.name ?? ''),
	);
	let targetLocation = $state(
		$appData.autoPop.enabled
			? ($appData.autoPop.converter.targetLocation ?? convState.target ?? '')
			: (convState.target ?? ''),
	);
	let createFolder = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.createNewFolder ?? true) : true,
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

	// Save state immediately when values change instead of in onDestroy
	$effect(() => {
		if (name !== null && name !== undefined) {
			wrapper(commands.convStateSet({ Name: name ?? '' }));
		}
	});

	$effect(() => {
		wrapper(commands.convStateSet({ CreateDirectory: createFolder }));
	});

	$effect(() => {
		if (targetLocation !== null && targetLocation !== undefined) {
			wrapper(commands.convStateSet({ Target: targetLocation }));
		}
	});
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={2} density="comfortable" rows="auto auto">
		<!-- Project Name -->
		<BentoItem glass>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFile size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Project Name`}</span>
			</HStack>

			<VStack gap="sm">
				<Input
					id="project-name"
					bind:value={name}
					placeholder={projectNamePlaceholder}
					style="seamless"
					onkeydown={(e) => {
						if (e.key === ' ') {
							e.stopPropagation();
						}
					}}
				/>
				<p class="text-muted text-xs">
					{$t`This will be used as the base name for your output files`}
				</p>
			</VStack>
		</BentoItem>

		<!-- Create Folder Toggle -->
		<BentoItem
			glass
			onclick={() => (createFolder = !createFolder)}
			data-keyhint={`enter;${$t`Toggle`}`}
		>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFolderPlus size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Output Structure`}</span>
			</HStack>

			<div
				class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
			>
				<VStack gap="xs" class="flex-1">
					<span class="text-sm font-medium">
						{createFolder ? $t`Create project folder` : $t`Direct output`}
					</span>
					<span class="text-muted text-xs">
						{#if createFolder}
							{$t`Files will be placed in a new folder named after the project`}
						{:else}
							{$t`Files will be saved directly in the target location`}
						{/if}
					</span>
				</VStack>
				<div class="pointer-events-none ml-4">
					<Toggle bind:checked={createFolder} tabindex={-1} style="seamless" />
				</div>
			</div>
		</BentoItem>

		<!-- Target Location -->
		<BentoItem colspan={2} glass onclick={select} data-keyhint={`enter;${$t`Select location`}`}>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFolder size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Output Location`}</span>
				{#if targetLocation}
					<div class="bg-success/20 ml-auto flex h-6 w-6 items-center justify-center rounded-full">
						<IconCheck size={14} class="text-success" />
					</div>
				{/if}
			</HStack>

			<VStack gap="sm">
				<div
					class="bg-surface-2 hover:bg-surface-1 group w-full cursor-pointer rounded-lg p-3 transition-colors"
				>
					<HStack gap="sm" align="center">
						<div
							class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {targetLocation
								? 'bg-accent-500/20'
								: 'bg-surface-1'}"
						>
							<IconFolder size={20} class={targetLocation ? 'text-accent-500' : 'text-muted'} />
						</div>
						<VStack gap="xs" class="flex-1 overflow-hidden">
							{#if targetLocation}
								<span class="text-sm font-medium">{$t`Location selected`}</span>
								<span class="text-muted truncate font-mono text-xs" title={targetLocation}>
									{truncatePath(targetLocation, 80)}
								</span>
							{:else}
								<span class="text-sm font-medium">{$t`No location selected`}</span>
								<span class="text-muted text-xs">{$t`Click to choose output folder`}</span>
							{/if}
						</VStack>
					</HStack>
				</div>
				<p class="text-muted text-xs">
					{$t`Choose where the converted files will be saved`}
				</p>
			</VStack>
		</BentoItem>
	</BentoGrid>
</div>
