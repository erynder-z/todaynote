<script lang="ts">
  /**
   * Component for choosing the application language from supported locales.
   */
  import { availableLocales, settings, t } from '$lib';

  /**
   * Updates the global locale setting when a new language is selected.
   */
  const handleLocaleChange = async (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const newLocale = target.value;
    await settings.save({ ...settings, locale: newLocale });
  };
</script>

<div class="setting-item">
  <label for="locale-select">{$t('settings.language')}</label>
  <select
    id="locale-select"
    class="theme-input"
    value={settings.locale}
    onchange={handleLocaleChange}
  >
    {#each $availableLocales as locale}
      <option value={locale.id}>{locale.name}</option>
    {/each}
  </select>
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    width: 100%;
  }

  label {
    font-weight: 600;
    color: var(--text-main);
  }

  .theme-input {
    -webkit-appearance: none;
    -moz-appearance: none;
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

  .theme-input:focus {
    background: color-mix(in srgb, var(--accent), transparent 85%);
  }
</style>
