<script lang="ts">
  import { settings, t } from '$lib';
  import { availableThemes } from '../utils/theme';

  const handleThemeChange = async (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const newTheme = target.value;
    await settings.save({ ...$settings, theme: newTheme });
  };
</script>

<div class="setting-item">
  <label for="theme-select">{$t('settings.theme')}</label>
  <select
    id="theme-select"
    class="theme-input"
    value={$settings.theme}
    onchange={handleThemeChange}
  >
    {#each $availableThemes as theme}
      <option value={theme.id}>{theme.name}</option>
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
