<script lang="ts">
  /**
   * Component for selecting thread visual representation style.
   * Allows users to choose between different visual styles for thread identification.
   */
  import { settings } from '$lib/stores/settings.svelte';
  import { t } from '$lib/utils/i18n';
  import IdentIconDotmatrix from './IdentIcon_dotmatrix.svelte';
  import IdentIconRound from './IdentIcon_round.svelte';

  /**
   * Updates the global thread visual style setting.
   */
  const handleStyleChange = async (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const newStyle = target.value as 'dotmatrix' | 'round' | 'none';
    await settings.set_identicon_style(newStyle);
  };
</script>

<div class="setting-item">
  <label for="thread-visual-select">{$t('settings.thread_visual_style')}</label>

  <select
    id="thread-visual-select"
    class="theme-input"
    value={settings.identiconStyle}
    onchange={handleStyleChange}
  >
    <option value="dotmatrix"
      >{$t('settings.thread_visual_identicon_dotmatrix')}</option
    >
    <option value="round">{$t('settings.thread_visual_identicon_round')}</option
    >
    <option value="none">{$t('settings.thread_visual_static_icon')}</option>
  </select>

  <!-- Preview of visual styles -->
  <div class="visual-preview-container">
    <div class="preview-item">
      <span class="preview-label"
        >{$t('settings.thread_visual_identicon_dotmatrix')}</span
      >
      <IdentIconDotmatrix title="Preview" size="1.5rem" />
    </div>

    <div class="preview-item">
      <span class="preview-label"
        >{$t('settings.thread_visual_identicon_round')}</span
      >
      <IdentIconRound title="Preview" size="1.5rem" />
    </div>

    <div class="preview-item">
      <span class="preview-label"
        >{$t('settings.thread_visual_static_icon')}</span
      >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="24"
        viewBox="0 -960 960 960"
        width="24"
        fill="currentColor"
        ><path
          d="M600-80v-100L320-320H120v-240h172l108-124v-196h240v240H468L360-516v126l240 120v-50h240v240H600Z"
        /></svg
      >
    </div>
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
    background: color-mix(in srgb, var(--accent), transparent 85%);
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

  .visual-preview-container {
    display: flex;
    gap: 1rem;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.5rem;
  }

  .preview-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
  }

  .preview-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: center;
  }

  @media (max-width: 600px) {
    .preview-item {
      flex: 0 0 30%;
    }
  }
</style>
