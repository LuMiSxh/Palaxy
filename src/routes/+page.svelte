<script lang="ts">
	import {
		IconHeadset,
		IconQuestionMark,
		IconSearch,
		IconSettings,
		IconTransform,
		IconUsers
	} from '@tabler/icons-svelte';

	import { goto } from '$app/navigation';

	// import { getContext } from 'svelte';
	// import { type ToastContext } from '@skeletonlabs/skeleton-svelte';
	//
	// export const toast: ToastContext = getContext('toast');
	//
	//
	// toast.create({
	// 	title: 'Error',
	// 	description: 'Uh oh, something went wrong.',
	// 	type: 'error'
	// });
</script>

<div class="grid h-full w-full select-none grid-cols-3 grid-rows-2 gap-10 p-3">
	<div
		role="button"
		onkeydown={() => _}
		tabindex="0"
		onclick={() => goto('/convert')}
		class="relative flex h-full w-full preset-tonal flex-col overflow-hidden rounded-xl p-3 shadow-lg transition-transform hover:scale-105 active:scale-95"
	>
		<div class="absolute inset-0 z-10">
			<div
				class="absolute right-0 top-0 h-2/3 w-2/3 bg-gradient-to-bl from-primary-500/80 to-transparent blur-2xl"
			></div>
		</div>
		<div class="z-20 ml-3 mt-3">
			<h1 class="text-2xl font-bold">Convert</h1>
			<p class="mr-3 mt-6">
				Convert your manga images into a digital format that can be read on your favorite devices.
			</p>
		</div>
		<IconTransform size="60" class="absolute right-3 top-3" />
	</div>
	<div
		role="button"
		onkeydown={() => _}
		tabindex="0"
		onclick={() => goto('/search')}
		class="relative flex h-full w-full preset-tonal flex-col overflow-hidden rounded-xl p-3 shadow-lg transition-transform hover:scale-105 active:scale-95"
	>
		<div class="absolute inset-0 z-10">
			<div
				class="absolute right-0 top-0 h-2/3 w-2/3 bg-gradient-to-bl from-tertiary-500/80 to-transparent blur-2xl"
			></div>
		</div>
		<div class="z-20 ml-3 mt-3">
			<h1 class="text-2xl font-bold">Search</h1>
			<p class="mr-3 mt-6">
				Search for your favorite manga series and chapters from various sources.
			</p>
		</div>
		<IconSearch size="60" class="absolute right-3 top-3" />
	</div>
	<div
		role="button"
		onkeydown={() => _}
		tabindex="0"
		onclick={() => goto('/agents')}
		class="relative flex h-full w-full preset-tonal flex-col overflow-hidden rounded-xl p-3 shadow-lg transition-transform hover:scale-105 active:scale-95"
	>
		<div class="absolute inset-0 z-10">
			<div
				class="absolute right-0 top-0 h-2/3 w-2/3 bg-gradient-to-bl from-secondary-400/80 to-transparent blur-2xl"
			></div>
		</div>
		<div class="z-20 ml-3 mt-3">
			<h1 class="text-2xl font-bold">Agents</h1>
			<p class="mr-3 mt-6">Manage your agents and their settings for better search results.</p>
		</div>
		<IconUsers size="60" class="absolute right-3 top-3" />
	</div>
	<div
		role="button"
		onkeydown={() => _}
		tabindex="0"
		onclick={() => goto('/settings')}
		class="relative flex h-full w-full preset-tonal flex-col overflow-hidden rounded-xl p-3 shadow-lg transition-transform hover:scale-105 active:scale-95"
	>
		<div class="absolute inset-0 z-10">
			<div
				class="absolute right-0 top-0 h-2/3 w-2/3 bg-gradient-to-bl from-slate-600/80 to-transparent blur-2xl"
			></div>
		</div>
		<div class="z-20 ml-3 mt-3">
			<h1 class="text-2xl font-bold">Settings</h1>
			<p class="mr-3 mt-6">Change Palaxy's settings to better suit your needs and preferences.</p>
		</div>
		<IconSettings size="60" class="absolute right-3 top-3" />
	</div>
	<div
		role="button"
		onkeydown={() => _}
		tabindex="0"
		onclick={() => goto('/learn-more')}
		class="relative flex h-full w-full preset-tonal flex-col overflow-hidden rounded-xl p-3 shadow-lg transition-transform hover:scale-105 active:scale-95"
	>
		<div class="absolute inset-0 z-10">
			<div
				class="absolute right-0 top-0 h-2/3 w-2/3 bg-gradient-to-bl from-blue-400/80 to-transparent blur-2xl"
			></div>
		</div>
		<div class="z-20 ml-3 mt-3">
			<h1 class="text-2xl font-bold">Learn More</h1>
			<p class="mr-3 mt-6">
				Learn how to use Palaxy and its features to get the most out of your experience.
			</p>
		</div>
		<IconQuestionMark size="60" class="absolute right-3 top-3" />
	</div>
	<div
		role="button"
		onkeydown={() => _}
		tabindex="0"
		onclick={() => goto('/support')}
		class="relative flex h-full w-full preset-tonal flex-col overflow-hidden rounded-xl p-3 shadow-lg transition-transform hover:scale-105 active:scale-95"
	>
		<div class="absolute inset-0 z-10">
			<div
				class="absolute right-0 top-0 h-2/3 w-2/3 bg-gradient-to-bl from-red-400/80 to-transparent blur-2xl"
			></div>
		</div>
		<div class="z-20 ml-3 mt-3">
			<h1 class="text-2xl font-bold">Support</h1>
			<p class="mr-3 mt-6">Need help? Found a bug? Have a suggestion? Contact us for support.</p>
		</div>
		<IconHeadset size="60" class="absolute right-3 top-3" />
	</div>
</div>
