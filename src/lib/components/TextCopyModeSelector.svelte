<script lang="ts">
  /**
   * Toggle for selecting between plain text and markdown copy mode.
   */
  import { settings } from '#lib/stores/settings.svelte.js';
  import { t } from '#lib/utils/i18n.js';

  $: options = [
    { value: 'plain', label: $t('settings.text_copy.plain') },
    { value: 'markdown', label: $t('settings.text_copy.markdown') },
  ];

  /**
   * Handle mode change.
   */
  const handleChange = async (event: Event) => {
    const select = event.target as HTMLSelectElement;
    const mode = select.value as 'plain' | 'markdown';
    await settings.saveTextCopyMode(mode);
  };
</script>

<div class="setting-card">
  <h3>{$t('settings.text_copy.title')}</h3>

  <select
    class="mode-selector"
    bind:value={settings.textCopyMode}
    on:change={handleChange}
    aria-label="Text copy mode"
  >
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
</div>

<style>
  .setting-card {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 1rem;
  }

  h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text-primary);
  }

  .mode-selector {
    -webkit-appearance: none;
    appearance: none;
    padding: 0.65rem 1rem;
    border-radius: 0.5rem;
    border: none;
    background: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--text-main);
    cursor: pointer;
    width: 25ch;
    font-size: 0.95rem;
    outline: none;
    text-align: center;
  }

  .mode-selector:focus {
    background: color-mix(in srgb, var(--accent), transparent 85%);
  }
</style>
