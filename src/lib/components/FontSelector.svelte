<script lang="ts">
  /**
   * Component for selecting and setting the application font
   */
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { settings, t } from '#lib';

  let systemFonts = $state<string[]>([]);
  let isLoading = $state(false);

  /**
   * Populates the system fonts list with available fonts
   */
  const loadSystemFonts = async () => {
    if (systemFonts.length > 0) return;

    try {
      isLoading = true;

      systemFonts = (await invoke<string[]>('get_system_fonts')).sort((a, b) =>
        a.localeCompare(b),
      );
    } catch (err) {
      console.error('Failed to load system fonts:', err);
      systemFonts = [];
    } finally {
      isLoading = false;
    }
  };

  /**
   * Handles selecting a custom font from the dropdown menu
   */
  const handleFontChange = async (e: Event) => {
    const selectElement = e.target as HTMLSelectElement;
    const selectedFont = selectElement.value;
    if (selectedFont === 'default') {
      await settings.saveFontFamily(null);
    } else {
      await settings.saveFontFamily(selectedFont);
    }
  };

  /**
   * Handles toggling the use custom font checkbox
   */
  const handleUseCustomFontToggle = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const useCustom = target.checked;
    await settings.saveUseCustomFont(useCustom);
    if (useCustom && systemFonts.length === 0) await loadSystemFonts();
  };

  onMount(() => {
    if (settings.useCustomFont && systemFonts.length === 0)
      loadSystemFonts().catch(console.error);
  });
</script>

<div class="setting-item">
  <label for="font-select">{$t('settings.font.select')}</label>
  <div class="font-container">
    <div class="font-name" hidden={settings.useCustomFont}>
      {$t('settings.font.defaultAppFont')}
    </div>
    <div class="input-container">
      <select
        id="font-select"
        class="theme-input"
        class:loading={isLoading}
        value={isLoading ? 'loading' : settings.fontFamily || 'default'}
        onchange={handleFontChange}
        hidden={!settings.useCustomFont}
        disabled={!settings.useCustomFont || isLoading}
      >
        {#if isLoading}
          <option value="loading" disabled>
            {$t('settings.font.loading')}
          </option>
        {:else}
          <option value="default">
            {$t('settings.font.defaultAppFont')}
          </option>

          {#each systemFonts as font}
            <option value={font}>{font}</option>
          {/each}
        {/if}
      </select>
    </div>
  </div>

  <div class="checkbox-container">
    <input
      type="checkbox"
      id="use-custom-font"
      checked={settings.useCustomFont}
      onchange={handleUseCustomFontToggle}
    />
    <label for="use-custom-font">{$t('settings.font.useCustom')}</label>
  </div>

  <div class="sentence">
    <span> The quick brown fox jumps over the lazy dog </span>
  </div>
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    width: 100%;
  }

  .font-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
  }

  .input-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
  }

  label {
    font-weight: 600;
    color: var(--text-main);
    cursor: pointer;
    user-select: none;
    text-align: center;
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
    transition: background 0.2s ease;
  }

  .theme-input:focus {
    background: color-mix(in srgb, var(--accent), transparent 85%);
  }

  .theme-input option {
    background-color: var(--bg-surface);
    color: var(--text-main);
  }

  .theme-input.loading option {
    color: var(--text-muted);
    font-style: italic;
  }

  .checkbox-container {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    cursor: pointer;
  }

  input[type='checkbox'] {
    width: 1.2rem;
    height: 1.2rem;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .sentence {
    font-size: 1rem;
    color: var(--text-muted);
    text-align: center;
    border-top: 1px solid var(--border);
    margin-top: 0.4rem;
    padding-top: 0.8rem;
  }
</style>
