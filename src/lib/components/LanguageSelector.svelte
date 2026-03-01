<script lang="ts">
  import { availableLocales, settings, t } from '$lib';

  const handleLocaleChange = async (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const newLocale = target.value;
    await settings.save({ ...$settings, locale: newLocale });
  };
</script>

<div class="setting-item">
  <label for="locale-select">{$t('settings.language')}</label>
  <select
    id="locale-select"
    class="theme-input"
    value={$settings.locale}
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
    padding: 0.8rem 1.5rem;
    border-radius: 0.5rem;
    border: 0.0625rem solid var(--border);
    background-color: var(--bg-surface);
    color: var(--text-main);
    cursor: pointer;
    width: 25ch;
  }
</style>
