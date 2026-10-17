<script lang="ts">
  /**
   * Component for toggling the floating toolbar visibility.
   */
  import { settings } from '../stores/settings.svelte.ts';
  import { t } from '../utils/i18n.ts';

  $: options = [
    { value: 'true', label: $t('settings.floating_toolbar.enabled') },
    { value: 'false', label: $t('settings.floating_toolbar.disabled') },
  ];

  /**
   * Updates the floating toolbar enabled state.
   */
  const handleChange = async (event: Event) => {
    const select = event.target as HTMLSelectElement;
    const enabled = select.value === 'true';
    await settings.saveFloatingToolbarEnabled(enabled);
  };
</script>

<div class="setting-card">
  <h3>{$t('settings.floating_toolbar')}</h3>
  <div
    class="toolbar-preview"
    class:disabled={!settings.floatingToolbarEnabled}
  >
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="M272-200v-560h221q65 0 120 40t55 111q0 51-23 78.5T602-491q25 11 55.5 41t30.5 90q0 89-65 124.5T501-200H272Zm121-112h104q48 0 58.5-24.5T566-372q0-11-10.5-35.5T494-432H393v120Zm0-228h93q33 0 48-17t15-38q0-24-17-39t-44-15h-95v109Z"
        /></svg
      >
    </button>
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="M200-200v-100h160l120-360H320v-100h400v100H580L460-300h140v100H200Z"
        /></svg
      >
    </button>
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="M80-400v-80h800v80H80Zm340-160v-120H200v-120h560v120H540v120H420Zm0 400v-160h120v160H420Z"
        /></svg
      >
    </button>
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="M320-240 80-480l240-240 57 57-184 184 183 183-56 56Zm320 0-57-57 184-184-183-183 56-56 240 240-240 240Z"
        /></svg
      >
    </button>
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="m228-240 92-160q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 23-5.5 42.5T458-480L320-240h-92Zm360 0 92-160q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 23-5.5 42.5T818-480L680-240h-92ZM362.5-517.5Q380-535 380-560t-17.5-42.5Q345-620 320-620t-42.5 17.5Q260-585 260-560t17.5 42.5Q295-500 320-500t42.5-17.5Zm360 0Q740-535 740-560t-17.5-42.5Q705-620 680-620t-42.5 17.5Q620-585 620-560t17.5 42.5Q655-500 680-500t42.5-17.5ZM680-560Zm-360 0Z"
        /></svg
      >
    </button>
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="M318-120q-82 0-140-58t-58-140q0-40 15-76t43-64l134-133 56 56-134 134q-17 17-25.5 38.5T200-318q0 49 34.5 83.5T318-200q23 0 45-8.5t39-25.5l133-134 57 57-134 133q-28 28-64 43t-76 15Zm79-220-57-57 223-223 57 57-223 223Zm251-28-56-57 134-133q17-17 25-38t8-44q0-50-34-85t-84-35q-23 0-44.5 8.5T558-726L425-592l-57-56 134-134q28-28 64-43t76-15q82 0 139.5 58T839-641q0 39-14.5 75T782-502L648-368Z"
        /></svg
      >
    </button>

    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="toolbar-btn" disabled>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.1rem"
        viewBox="0 -960 960 960"
        width="1.1rem"
        fill="currentColor"
        ><path
          d="M360-240q-33 0-56.5-23.5T280-320v-480q0-33 23.5-56.5T360-880h360q33 0 56.5 23.5T800-800v480q0 33-23.5 56.5T720-240H360Zm0-80h360v-480H360v480ZM200-80q-33 0-56.5-23.5T120-160v-560h80v560h440v80H200Zm160-240v-480 480Z"
        /></svg
      >
    </button>
  </div>

  <select
    class="toolbar-selector"
    value={settings.floatingToolbarEnabled ? 'true' : 'false'}
    on:change={handleChange}
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
    width: 100%;
  }

  h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text-primary);
  }

  .toolbar-preview {
    display: flex;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    padding: 0.75rem;
    background: var(--bg-surface-variant);
    box-shadow:
      0 10px 30px -5px rgba(0, 0, 0, 0.2),
      0 4px 12px -4px rgba(0, 0, 0, 0.1);
    border-radius: 0.5rem;
    margin: 0.5rem 0;
    transition: opacity 0.2s ease;
  }

  .toolbar-preview.disabled {
    opacity: 0.35;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-ui-muted, var(--text-muted));
    cursor: pointer;
    transition:
      background-color 0.2s cubic-bezier(0.4, 0, 0.2, 1),
      color 0.2s,
      transform 0.1s ease;
    padding: 0;
  }

  .toolbar-btn:disabled {
    cursor: default;
    opacity: 1;
  }

  .toolbar-selector {
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

  .toolbar-selector:focus {
    background: color-mix(in srgb, var(--accent), transparent 85%);
  }
</style>
