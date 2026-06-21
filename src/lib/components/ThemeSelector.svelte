<script lang="ts">
  /**
   * Component for selecting the application's visual theme.
   */
  import { settings, t } from '$lib';
  import { availableThemes } from '../utils/theme';

  /**
   * Updates the global theme setting and applies new colors to the UI.
   */
  const handleThemeChange = async (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const newTheme = target.value;
    await settings.save({ ...settings, theme: newTheme });
  };
</script>

<div class="setting-item">
  <label for="theme-select">{$t('settings.theme')}</label>
  <select
    id="theme-select"
    class="theme-input"
    value={settings.theme}
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
    background: color-mix(in srgb, var(--accent), transparent 75%);
  }

  .theme-input option {
    background-color: var(--bg-surface);
    color: var(--text-main);
  }
</style>
