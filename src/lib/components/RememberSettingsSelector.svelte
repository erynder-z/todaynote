<script lang="ts">
  /**
   * Component for toggling the persistence of component-specific settings
   * (like search mode, layout choice, etc.) across sessions.
   */
  import { settings } from '../stores/settings.svelte';
  import { t } from '../utils/i18n';

  /**
   * Updates the global setting that determines whether transient component
   * states are saved or reset to defaults on every mount.
   */
  const handleToggle = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const remember = target.checked;
    await settings.save({
      ...settings,
      rememberSettings: remember,
    });
  };
</script>

<div class="setting-item">
  <div class="checkbox-container">
    <input
      type="checkbox"
      id="remember-settings"
      checked={settings.rememberSettings}
      onchange={handleToggle}
    />
    <label for="remember-settings">{$t('settings.remember_settings')}</label>
  </div>
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
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

  label {
    font-weight: 600;
    color: var(--text-main);
    cursor: pointer;
    user-select: none;
  }
</style>
