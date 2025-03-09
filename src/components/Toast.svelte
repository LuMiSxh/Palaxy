<script lang="ts">
	import toaster from '$states/toast.svelte';
	import { IconX } from '@tabler/icons-svelte';
	import { slide  } from 'svelte/transition';

	function getAlertType(type: 'info' | 'success' | 'warning' | 'error') {
		switch (type) {
			case 'info':
				return 'alert-info';
			case 'success':
				return 'alert-success';
			case 'warning':
				return 'alert-warning';
			case 'error':
				return 'alert-error';
			default:
				return 'alert-info';
		}
	}

	function getBtnType(type: 'info' | 'success' | 'warning' | 'error') {
		switch (type) {
			case 'info':
				return 'btn-info';
			case 'success':
				return 'btn-success';
			case 'warning':
				return 'btn-warning';
			case 'error':
				return 'btn-error';
			default:
				return 'btn-info';
		}
	}
</script>

<div class="fixed flex flex-col gap-2 right-4 top-4 z-50">
	{#each toaster.toasts as toast}
		<div class="alert {getAlertType(toast.type)} flex justify-between items-center" transition:slide>
			<span class="mr-2">
				<!--eslint-disable-next-line svelte/no-at-html-tags-->
				{@html toast.message}
			</span>
			<button
				class="btn btn-soft btn-xs {getBtnType(toast.type)}"
				onclick={() => toaster.removeToast(toast.id)}
			>
				<IconX />
			</button>
		</div>
	{/each}
</div>
