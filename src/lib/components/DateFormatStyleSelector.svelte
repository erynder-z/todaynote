<script lang="ts">
  /**
   * Toggle for selecting between long and narrow date formats.
   * Medium format: "Mon, January 01, 2026"
   * Narrow format: "M, Jan 01, 26"
   */
  import { settings } from '$lib/stores/settings.svelte';
  import { locale, t } from '$lib/utils/i18n';

  $: options = [
    { value: 'medium', label: $t('settings.date.medium') },
    { value: 'narrow', label: $t('settings.date.narrow') },
  ];

  /**
   * Handle format style change.
   */
  const handleChange = async (event: Event) => {
    const select = event.target as HTMLSelectElement;
    const style = select.value as 'medium' | 'narrow';
    await settings.saveDateFormatStyle(style);
  };

  /**
   * Get a preview of how dates will look in each format.
   */
  $: mediumPreview = new Intl.DateTimeFormat($locale, {
    weekday: 'short',
    year: 'numeric',
    month: 'long',
    day: '2-digit',
  }).format(new Date());

  $: narrowPreview = new Intl.DateTimeFormat($locale, {
    weekday: 'narrow',
    year: '2-digit',
    month: 'short',
    day: '2-digit',
  }).format(new Date());
</script>

<div class="setting-card">
  <h3>{$t('settings.date.title')}</h3>

  <div class="format-preview">
    <div class="preview-option">
      <strong>{$t('settings.date.medium')}</strong>
      {mediumPreview}
    </div>
    <div class="preview-option">
      <strong>{$t('settings.date.narrow')}</strong>
      {narrowPreview}
    </div>
  </div>

  <select
    class="format-selector"
    bind:value={settings.dateFormatStyle}
    on:change={handleChange}
    aria-label="Date format style"
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

  .format-preview {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    padding: 0.75rem;
    background: var(--bg-surface-variant);
    border-radius: 0.5rem;
    margin: 0.5rem 0;
  }

  .preview-option {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .preview-option strong {
    color: var(--text-primary);
    min-width: 60px;
  }

  .format-selector {
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

  .format-selector:focus {
    background: color-mix(in srgb, var(--accent), transparent 85%);
  }
</style>
