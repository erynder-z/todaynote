<script lang="ts">
  /**
   * Component for choosing the custom default thread name for new notes.
   */
  import { settings, t } from '$lib';

  let localValue = $state(settings.defaultThreadName || '');

  /**
   * Derived value that shows the effective thread name (custom or localized default).
   */
  let effectiveValue = $derived(
    settings.defaultThreadName || $t('note.header'),
  );

  /**
   * Updates the global setting when the input changes.
   */
  const handleValueChange = async () => {
    const newValue = localValue.trim() || null;
    await settings.save({ ...settings, defaultThreadName: newValue });
  };

  /**
   * Sync local value with settings when settings change externally.
   */
  $effect(() => {
    localValue = settings.defaultThreadName || '';
  });
</script>

<div class="setting-item">
  <label for="default-thread-name"
    >{$t('settings.default_thread_name.title')}</label
  >
  <input
    id="default-thread-name"
    type="text"
    class="theme-input"
    placeholder={effectiveValue}
    bind:value={localValue}
    onchange={handleValueChange}
  />
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
    padding: 0.65rem 1rem;
    border-radius: 0.5rem;
    border: none;
    background: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--text-main);
    width: 25ch;
    font-size: 0.95rem;
    outline: none;
    transition: background 0.2s ease;
  }

  .theme-input:focus {
    background: color-mix(in srgb, var(--accent), transparent 75%);
  }

  .theme-input::placeholder {
    color: color-mix(in srgb, var(--text-main), transparent 50%);
  }
</style>
