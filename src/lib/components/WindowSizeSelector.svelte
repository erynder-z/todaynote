<script lang="ts">
  import { settings, t } from '$lib';

  const handleToggle = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const remember = target.checked;
    await settings.save({ ...$settings, remember_window_size: remember });
  };
</script>

<div class="setting-item">
  <div class="checkbox-container">
    <input
      type="checkbox"
      id="remember-window-size"
      checked={$settings.remember_window_size}
      onchange={handleToggle}
    />
    <label for="remember-window-size"
      >{$t('settings.remember_window_size')}</label
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
