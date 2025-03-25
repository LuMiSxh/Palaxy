<script lang="ts">
	import { IconLoader2 } from '@tabler/icons-svelte';
	import { t } from 'svelte-i18n-lingui';

	let { text = $bindable($t`Loading...`) }: { text: string } = $props();
</script>

<div class="flex flex-col items-center justify-center py-8">
	<div class="mb-4 animate-spin">
		<IconLoader2 size={32} class="text-primary" />
	</div>
	<p class="text-content-secondary dark:text-content-dark-secondary">
		{text}
	</p>
</div>
