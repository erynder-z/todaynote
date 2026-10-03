<script lang="ts">
  /**
   * Component for toggling whether the user can manually create notes.
   */
  import { settings } from '../stores/settings.svelte';
  import { t } from '../utils/i18n';

  let isActive = $derived(settings.allowManualNoteCreation);

  /**
   * Updates the global setting that determines whether manual note
   * creation is allowed.
   */
  const handleToggle = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const allow = target.checked;
    await settings.save({
      ...settings,
      allowManualNoteCreation: allow,
    });
  };
</script>

<div class="setting-item">
  <div class="checkbox-container">
    <input
      type="checkbox"
      id="allow-manual-note-creation"
      checked={settings.allowManualNoteCreation}
      onchange={handleToggle}
    />
    <label for="allow-manual-note-creation"
      >{$t('settings.allow_manual_note_creation')}</label
    >
  </div>
  <div class="icon-container {!isActive ? 'muted-color' : 'normal-color'}">
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="2rem"
      viewBox="0 -960 960 960"
      width="2rem"
      fill="currentColor"
      ><path
        d="M440-240h80v-120h120v-80H520v-120h-80v120H320v80h120v120ZM240-80q-33 0-56.5-23.5T160-160v-640q0-33 23.5-56.5T240-880h320l240 240v480q0 33-23.5 56.5T720-80H240Zm280-520v-200H240v640h480v-440H520ZM240-800v200-200 640-640Z"
      /></svg
    >
  </div>
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
  }

  .icon-container {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: 1rem;
    transition: opacity 0.2s ease;
  }

  .normal-color {
    color: var(--text-ui-muted);
  }

  .muted-color {
    opacity: 0.35;
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
