<script lang="ts">
	import { FeatureFlag } from '$types/appdata';
	import { msg, t } from 'svelte-i18n-lingui';
	import { appData } from '$stores/appdata';
	import { addToast } from '$states/toast.svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import dialogManager from '$states/dialog.svelte';

	let { id = $bindable(null) } = $props();

	let selectedFeatures = $state($appData.featureFlags);

	// Create a map of feature flags to their display names and descriptions
	// Possible types: 'experimental', 'beta', 'stable'
	const featureMap = {
		[FeatureFlag.CHANGE_LANGUAGE]: {
			name: $t`Language Settings`,
			description: $t`Allows changing the application language`,
			type: msg`beta`
		},
		[FeatureFlag.SEARCH_MANGA]: {
			name: $t`Manga Search`,
			description: $t`Enable manga search functionality`,
			type: msg`experimental`
		},
		[FeatureFlag.BROWSE_AGENTS]: {
			name: $t`Browse Agents`,
			description: $t`Add browsing agents feature`,
			type: msg`experimental`
		},
		[FeatureFlag.MOUSE_SUPPORT]: {
			name: $t`Mouse Support`,
			description: $t`Enable the mouse support feature for the application`,
			type: msg`stable`
		},
		[FeatureFlag.CUSTOM_KEYBINDS]: {
			name: $t`Custom Keybinds`,
			description: $t`Allow setting custom keybinds for the application`,
			type: msg`experimental`
		}
	};

	// All available features as an array
	const availableFeatures = Object.values(FeatureFlag).filter(
		(value) => typeof value === 'number'
	) as FeatureFlag[];

	// Toggle a feature selection
	function toggleFeature(feature: FeatureFlag) {
		if (selectedFeatures.includes(feature)) {
			selectedFeatures = selectedFeatures.filter((f) => f !== feature);
		} else {
			selectedFeatures = [...selectedFeatures, feature];
		}
	}

	function save() {
		// Save the selected features
		$appData.featureFlags = selectedFeatures;

		addToast($t`Feature flags saved`, 'success');
		dialogManager.closeDialog(id);
	}

	function cancel() {
		// Reset the selected features
		addToast($t`Feature flags are not saved`, 'warning');
		dialogManager.closeDialog(id);
	}

	// use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
</script>

<div {id}>
	<p class="text-content-secondary dark:text-content-dark-secondary mb-4">
		{$t`Enable or disable application features`}
	</p>

	<div class="space-y-3">
		{#each availableFeatures as feature}
			<div class="fieldset">
				<div class="flex items-center justify-between">
					<div>
						<div class="flex items-center gap-2">
							<h3>{featureMap[feature].name}</h3>
							<span
								class={`badge badge-sm ${
									featureMap[feature].type === 'experimental'
										? 'badge-error'
										: featureMap[feature].type === 'beta'
											? 'badge-warning'
											: 'badge-success'
								}`}
							>
								{$t(featureMap[feature].type)}
							</span>
						</div>
						<p class="text-content-secondary dark:text-content-dark-secondary text-sm">
							{featureMap[feature].description}
						</p>
					</div>

					<label class="toggle toggle-lg">
						<input
							type="checkbox"
							class="toggle-input"
							use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
							onkeydown={(evt) => {
								if (evt.key === 'Enter') {
									evt.preventDefault();
									evt.stopPropagation();

									toggleFeature(feature);
								}
							}}
							checked={selectedFeatures.includes(feature)}
							onchange={() => toggleFeature(feature)}
						/>
						<span class="toggle-track">
							<span class="toggle-thumb"></span>
						</span>
					</label>
				</div>
			</div>
		{/each}
	</div>

	<div class="mt-6 flex justify-end space-x-3">
		<button
			class="btn btn-outline"
			onclick={cancel}
			use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
			onkeydown={(evt) => {
				if (evt.key === 'Enter') {
					evt.preventDefault();
					evt.stopPropagation();

					cancel();
				}
			}}
		>
			{$t`Cancel`}
		</button>
		<button
			class="btn btn-primary"
			onclick={save}
			use:handleKeyHint={{ keys: [['enter', $t`Invoke`]] }}
			onkeydown={(evt) => {
				if (evt.key === 'Enter') {
					evt.preventDefault();
					evt.stopPropagation();

					save();
				}
			}}
		>
			{$t`Save`}
		</button>
	</div>
</div>
