<script lang="ts">
  import { settings, t } from '$lib';

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
    class="locale-select"
    value={$settings.locale}
    onchange={handleLocaleChange}
  >
    <option value="en">English</option>
    <option value="de">Deutsch</option>
    <option value="jp">日本語</option>
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
    font-size: 1.1rem;
  }

  .locale-select {
    padding: 0.8rem 1.5rem;
    border-radius: 8px;
    border: 1px solid #ccc;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
    width: 200px;
  }

  @media (prefers-color-scheme: dark) {
    .locale-select {
      background-color: #2a2a2a;
      color: white;
      border-color: #444;
    }
  }
</style>
