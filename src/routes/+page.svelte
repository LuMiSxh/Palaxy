<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { BentoGrid, BentoItem, VStack, HStack, Separator } from 'waku/layout';
	import { Button, Badge, LoadingSpinner } from 'waku/components';
	import { IconTransform, IconCommand } from '@tabler/icons-svelte';
	import { keyHint } from '$states/keyhint.svelte';
	import { getVersion, getTauriVersion } from '@tauri-apps/api/app';

	onMount(async () => {
		// Global ActionHub hint (Layer 0)
		keyHint.clear();
		keyHint.addKey('space', $t`ActionHub`);
	});

	// Layer 1: Navigation Hints
	// Automatically removed when this page unmounts
	$effect(() => {
		return keyHint.register([
			['tab', $t`Navigate`],
			['shift+tab', $t`Navigate`],
		]);
	});

	async function getVersions() {
		const appVer = await getVersion();
		const tauriVer = await getTauriVersion();
		return { appVer, tauriVer };
	}

	function openActionHub() {
		window.dispatchEvent(new KeyboardEvent('keydown', { key: ' ' }));
		window.dispatchEvent(new KeyboardEvent('keyup', { key: ' ' }));
	}
</script>

<div class="h-full w-full overflow-y-auto p-3 md:p-6">
	<VStack gap="md" class="mx-auto h-full">
		<!-- Header -->
		<div class="flex items-center justify-between gap-4">
			<HStack gap="sm" align="center">
				<h1
					class="from-accent-500 to-accent-300 bg-linear-to-r bg-clip-text text-2xl font-bold text-transparent"
				>
					Palaxy
				</h1>
				<div class="bg-surface-2 h-4 w-px"></div>
				<p class="text-muted text-sm">{$t`Manga conversion and search`}</p>
			</HStack>
		</div>

		<Separator class="my-0!" />

		<!-- Dashboard Grid -->
		<BentoGrid cols={3} density="comfortable" class="flex-1">
			<!-- Primary Action: Convert -->
			<BentoItem
				colspan={2}
				rowspan={2}
				glass
				class="group relative min-h-[300px] overflow-hidden outline-none"
				onclick={() => goto('/convert')}
				data-keyhint={`enter;${$t`Invoke`}`}
			>
				<div
					class="from-accent-500/10 absolute inset-0 bg-linear-to-br to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100"
				></div>

				<div class="relative z-10 flex h-full flex-col justify-between p-1">
					<VStack gap="sm" align="start">
						<Badge variant="primary" class="text-xs">{$t`Ready`}</Badge>
						<div>
							<h2 class="text-xl font-bold">{$t`Convert Manga`}</h2>
							<p class="text-muted mt-1 line-clamp-2 max-w-sm text-sm">
								{$t`Convert your manga images into a digital format (EPUB/CBZ).`}
							</p>
						</div>
					</VStack>

					<Button
						size="md"
						variant="primary"
						onclick={(e) => {
							e.stopPropagation();
							goto('/convert');
						}}
						style="seamless"
						class="shadow-accent-500/20 mt-2 self-start shadow-lg"
						tabindex={-1}
						data-keyhint={`enter;${$t`Invoke`}`}
					>
						<IconTransform size={18} />
						{$t`Start Conversion`}
					</Button>
				</div>

				<!-- Character Art Decoration -->
				<img
					src="/holo/watame.png"
					alt="Character"
					class="pointer-events-none absolute -right-6 -bottom-10 h-[125%] object-contain opacity-20 grayscale transition-all
                           duration-500 group-focus-within:scale-105 group-focus-within:rotate-3 group-focus-within:opacity-60
                           group-focus-within:grayscale-0 group-hover:scale-105 group-hover:rotate-3 group-hover:opacity-60
                           group-hover:grayscale-0"
				/>
			</BentoItem>

			<!-- Quick Action: ActionHub -->
			<BentoItem
				onclick={openActionHub}
				data-keyhint={`enter;${$t`Invoke`}`}
				class="group hover:border-accent-500/50 focus:border-accent-500/50 cursor-pointer items-center justify-center text-center transition-colors outline-none"
			>
				<VStack align="center" gap="sm">
					<div
						class="bg-surface-2 group-hover:bg-accent-500 rounded-full p-3 shadow-sm transition-all duration-300 group-hover:text-white"
					>
						<IconCommand size={24} />
					</div>
					<VStack gap="none">
						<h3 class="text-sm font-bold">Action Hub</h3>
						<span class="text-muted text-[10px] tracking-wider uppercase">Space</span>
					</VStack>
				</VStack>
			</BentoItem>

			<!-- System Status -->
			<BentoItem class="justify-between">
				<HStack justify="between" align="center">
					<h3 class="text-sm font-semibold">System</h3>
					<div
						class="h-1.5 w-1.5 animate-pulse rounded-full bg-green-300 shadow-[0_0_8px_rgba(34,197,94,0.6)]"
					></div>
				</HStack>
				{#await getVersions()}
					<LoadingSpinner size="sm" />
				{:then versions}
					<div class="mt-2 grid grid-cols-2 gap-2">
						<div class="bg-surface-2 rounded-lg p-2 pl-3">
							<div class="text-muted text-[10px] uppercase">Tauri</div>
							<div class="font-mono text-sm font-bold">v{versions.tauriVer}</div>
						</div>
						<div class="bg-surface-2 rounded-lg p-2 pl-3">
							<div class="text-muted text-[10px] uppercase">App</div>
							<div class="font-mono text-sm font-bold">v{versions.appVer}</div>
						</div>
					</div>
				{:catch _err}
					<div class="text-sm text-red-500">Error loading version</div>
				{/await}
			</BentoItem>
		</BentoGrid>
	</VStack>
</div>
