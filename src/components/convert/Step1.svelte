<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import convState from '$states/converter.svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import {
		IconFolder,
		IconFolderOpen,
		IconCheck,
		IconFileZip,
	} from '@tabler/icons-svelte';

	import { keyboard } from '$lib/keyboard';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Badge } from 'waku/components';

	let unregisterKeyboard: () => void;
	let isZipSource = $state(false);

	onMount(async () => {
		// Reset
		await wrapper(commands.convStateReset());
		converter.reset();

		// Set Keyboard
		unregisterKeyboard = keyboard.smartRegister([
			['enter', selectFolder],
			['shift+enter', selectZip],
		]);

		// Auto-focus the BentoItem for keyboard-first navigation
		setTimeout(() => {
			const bentoItem = document.querySelector('.bento-item[data-keyhint]') as HTMLElement;
			bentoItem?.focus();
		}, 100);
	});

	onDestroy(() => {
		if (unregisterKeyboard) unregisterKeyboard();
	});

	async function selectFolder(evt: KeyboardEvent | MouseEvent | undefined = undefined) {
		evt?.stopPropagation();
		evt?.preventDefault();

		convState.source =
			(await open({
				directory: true,
				multiple: false,
			})) ?? '';

		if (converter.source !== null) {
			isZipSource = false;
			await wrapper(commands.convStateSet({ Source: convState.source ?? '' }));
		}
	}

	async function selectZip(evt: KeyboardEvent | MouseEvent | undefined = undefined) {
		evt?.stopPropagation();
		evt?.preventDefault();

		convState.source =
			(await open({
				directory: false,
				multiple: false,
				filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
			})) ?? '';

		if (converter.source !== null) {
			isZipSource = true;
			await wrapper(commands.convStateSet({ Source: convState.source ?? '' }));
		}
	}
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={3} density="comfortable" rows="auto 1fr" class="h-full">
		<!-- Summary Cards Row -->
		<BentoItem glass>
			<HStack gap="md" align="center">
				<div
					class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {convState.source
						? 'bg-success/10'
						: 'bg-surface-2'}"
				>
					<IconCheck size={20} class={convState.source ? 'text-success' : 'text-muted'} />
				</div>
				<VStack gap="none">
					<span class="text-muted text-xs font-medium tracking-wide uppercase">{$t`Status`}</span>
					<div
						class="text-2xl leading-tight font-bold {convState.source
							? 'text-success'
							: 'text-muted'}"
					>
						{convState.source ? $t`Ready` : $t`Waiting`}
					</div>
				</VStack>
			</HStack>
		</BentoItem>

		<BentoItem glass colspan={2}>
			<HStack gap="md" align="center">
				<div
					class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full {convState.source
						? 'bg-accent-500/10'
						: 'bg-surface-2'}"
				>
					{#if isZipSource}
						<IconFileZip size={20} class={convState.source ? 'text-accent-500' : 'text-muted'} />
					{:else}
						<IconFolderOpen size={20} class={convState.source ? 'text-accent-500' : 'text-muted'} />
					{/if}
				</div>
				<VStack gap="none">
					<span class="text-muted text-xs font-medium tracking-wide uppercase">{$t`Source`}</span>
					<div class="text-2xl leading-tight font-bold">
						{#if convState.source}
							{isZipSource ? $t`ZIP` : $t`Folder`}
						{:else}
							{$t`None`}
						{/if}
					</div>
				</VStack>
			</HStack>
		</BentoItem>

		<!-- Main Source Selection Card -->
		<BentoItem
			colspan={2}
			glass
			onclick={selectFolder}
			onkeydown={(e) => {
				if (e.key === ' ') {
					e.preventDefault();
					e.stopPropagation();
				}
			}}
			data-keyhint={`enter;${$t`Select source folder`}`}
			class="flex min-h-0 flex-col"
		>
			<HStack gap="sm" align="center" class="text-muted mb-3 shrink-0">
				<IconFolder size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Source Folder`}</span>
				{#if convState.source && !isZipSource}
					<div class="bg-success/20 ml-auto flex h-6 w-6 items-center justify-center rounded-full">
						<IconCheck size={14} class="text-success" />
					</div>
				{/if}
			</HStack>

			{#if convState.source && !isZipSource}
				<VStack gap="sm">
					<div
						class="bg-surface-2 hover:bg-surface-1 group w-full cursor-pointer rounded-lg p-3 transition-colors"
					>
						<HStack gap="sm" align="center">
							<div
								class="bg-accent-500/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
							>
								<IconFolderOpen size={20} class="text-accent-500" />
							</div>
							<VStack gap="xs" class="flex-1 overflow-hidden">
								<span class="text-sm font-medium">{$t`Folder selected`}</span>
								<span class="text-muted truncate font-mono text-xs" title={convState.source}>
									{truncatePath(convState.source, 80)}
								</span>
							</VStack>
						</HStack>
					</div>
					<p class="text-muted text-xs">
						{$t`Click to change the source folder or press Next to analyze`}
					</p>
				</VStack>
			{:else if !convState.source}
				<div class="flex h-full items-center justify-center p-8">
					<VStack gap="md" align="center" class="max-w-md text-center">
						<div class="bg-surface-2 flex h-16 w-16 items-center justify-center rounded-full">
							<IconFolder size={32} class="text-muted" />
						</div>
						<VStack gap="xs" align="center">
							<h3 class="text-lg font-semibold">{$t`Select Folder`}</h3>
							<p class="text-muted text-sm">
								{$t`Choose a directory containing your manga images`}
							</p>
						</VStack>
						<div class="text-muted flex items-center gap-2 text-xs">
							<Badge variant="primary" class="text-xs">⏎ {$t`Enter`}</Badge>
							<span>{$t`or click to browse`}</span>
						</div>
					</VStack>
				</div>
			{/if}
		</BentoItem>

		<!-- ZIP Source Selection Card -->
		<BentoItem
			glass
			onclick={selectZip}
			data-keyhint={`shift+enter;${$t`Select ZIP file`}`}
			class="flex min-h-0 flex-col"
		>
			<HStack gap="sm" align="center" class="text-muted mb-3 shrink-0">
				<IconFileZip size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`ZIP Archive`}</span>
				{#if convState.source && isZipSource}
					<div class="bg-success/20 ml-auto flex h-6 w-6 items-center justify-center rounded-full">
						<IconCheck size={14} class="text-success" />
					</div>
				{/if}
			</HStack>

			{#if convState.source && isZipSource}
				<VStack gap="sm">
					<div
						class="bg-surface-2 hover:bg-surface-1 group w-full cursor-pointer rounded-lg p-3 transition-colors"
					>
						<HStack gap="sm" align="center">
							<div
								class="bg-accent-500/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
							>
								<IconFileZip size={20} class="text-accent-500" />
							</div>
							<VStack gap="xs" class="flex-1 overflow-hidden">
								<span class="text-sm font-medium">{$t`ZIP selected`}</span>
								<span class="text-muted truncate font-mono text-xs" title={convState.source}>
									{truncatePath(convState.source, 80)}
								</span>
							</VStack>
						</HStack>
					</div>
					<p class="text-muted text-xs">
						{$t`Click to change the ZIP file or press Next to analyze`}
					</p>
				</VStack>
			{:else if !convState.source}
				<div class="flex h-full items-center justify-center p-8">
					<VStack gap="md" align="center" class="max-w-md text-center">
						<div class="bg-surface-2 flex h-16 w-16 items-center justify-center rounded-full">
							<IconFileZip size={32} class="text-muted" />
						</div>
						<VStack gap="xs" align="center">
							<h3 class="text-lg font-semibold">{$t`Select ZIP`}</h3>
							<p class="text-muted text-sm">
								{$t`Choose a ZIP archive containing your manga images`}
							</p>
						</VStack>
						<div class="text-muted flex items-center gap-2 text-xs">
							<Badge variant="primary" class="text-xs">⇧⏎ {$t`Shift+Enter`}</Badge>
							<span>{$t`or click to browse`}</span>
						</div>
					</VStack>
				</div>
			{/if}
		</BentoItem>
	</BentoGrid>
</div>
