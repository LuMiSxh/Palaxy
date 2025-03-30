<script lang="ts">
	import toaster from '$states/toast.svelte';
	import {
		IconAlertCircle,
		IconAlertTriangle,
		IconCircleCheck,
		IconInfoCircle,
		IconX
	} from '@tabler/icons-svelte';
	import { fade, fly } from 'svelte/transition';

	function getToastIcon(type: 'info' | 'success' | 'warning' | 'error') {
		switch (type) {
			case 'info':
				return IconInfoCircle;
			case 'success':
				return IconCircleCheck;
			case 'warning':
				return IconAlertTriangle;
			case 'error':
				return IconAlertCircle;
			default:
				return IconInfoCircle;
		}
	}

	function getToastColor(type: 'info' | 'success' | 'warning' | 'error') {
		switch (type) {
			case 'info':
				return 'stroke-info';
			case 'success':
				return 'stroke-success';
			case 'warning':
				return 'stroke-warning';
			case 'error':
				return 'stroke-error';
			default:
				return 'stroke-info';
		}
	}
</script>

<div class="fixed top-4 right-4 z-50 flex w-80 flex-col gap-3">
	{#each toaster.toasts as toast}
		<div
			class="surface flex items-center gap-3 p-3 pr-2 shadow-md"
			in:fly={{ y: -20, duration: 300 }}
			out:fade={{ duration: 200 }}
		>
			<div class="flex flex-shrink-0 items-center">
				<svelte:component
					this={getToastIcon(toast.type)}
					size={20}
					class={getToastColor(toast.type)}
				/>
			</div>

			<div class="flex min-w-0 flex-1 items-center">
				<!--eslint-disable-next-line svelte/no-at-html-tags-->
				{@html toast.message}
			</div>

			<button
				class="btn btn-soft btn-soft text-content-tertiary dark:text-content-dark-tertiary hover:text-content hover:dark:text-content-dark flex-shrink-0 rounded-full p-1 transition-colors"
				on:click={() => toaster.removeToast(toast.id)}
				aria-label="Close notification"
			>
				<IconX size={16} class="stroke-error" />
			</button>
		</div>
	{/each}
</div>
