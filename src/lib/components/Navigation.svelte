<script lang="ts">
  /**
   * Navigation bar that provides access to the several app features.
   */
  import type { PopupType } from '$lib/types/ui';
  import { sessionState } from '../stores/sessionState.svelte';
  import { t } from '../utils/i18n';
  import { useShortcuts } from '../utils/shortcuts';

  /**
   * Toggles the visibility of a specific popup by updating the global state.
   */
  const togglePopup = (type: PopupType) => {
    sessionState.activePopup = sessionState.activePopup === type ? null : type;
  };

  useShortcuts({
    toggleSearch: () => togglePopup('search'),
    toggleNoteBrowser: () => togglePopup('noteBrowser'),
    toggleSettings: () => togglePopup('folderSelector'),
  });
</script>

<nav class="navigation">
  <button
    onclick={() => togglePopup('noteBrowser')}
    class="nav-icon"
    title={$t('navigation.home')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.25rem"
      viewBox="0 -960 960 960"
      width="1.25rem"
      fill="currentColor"
      ><path
        d="M480-400 40-640l440-240 440 240-440 240Zm0 160L63-467l84-46 333 182 333-182 84 46-417 227Zm0 160L63-307l84-46 333 182 333-182 84 46L480-80Zm0-411 273-149-273-149-273 149 273 149Zm0-149Z"
      /></svg
    >
  </button>

  <button
    onclick={() => togglePopup('search')}
    class="nav-icon"
    title={$t('navigation.search')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.25rem"
      viewBox="0 -960 960 960"
      width="1.25rem"
      fill="currentColor"
      ><path
        d="M784-120 532-372q-30 24-69 38t-83 14q-109 0-184.5-75.5T120-580q0-109 75.5-184.5T380-840q109 0 184.5 75.5T640-580q0 44-14 83t-38 69l252 252-56 56ZM380-400q75 0 127.5-52.5T560-580q0-75-52.5-127.5T380-760q-75 0-127.5 52.5T200-580q0 75 52.5 127.5T380-400Z"
      /></svg
    >
  </button>

  <button
    onclick={() => togglePopup('folderSelector')}
    class="nav-icon"
    title={$t('navigation.settings')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.25rem"
      viewBox="0 -960 960 960"
      width="1.25rem"
      fill="currentColor"
      ><path
        d="m370-80-16-128q-13-5-24.5-12T307-235l-119 50L78-375l103-78q-1-7-1-13.5v-27q0-6.5 1-13.5L78-585l110-190 119 50q11-8 23-15t24-12l16-128h220l16 128q13 5 24.5 12t22.5 15l119-50 110 190-103 78q1 7 1 13.5v27q0 6.5-2 13.5l103 78-110 190-118-50q-11 8-23 15t-24 12L590-80H370Zm70-80h79l14-106q31-8 57.5-23.5T639-327l99 41 39-68-86-65q5-14 7-29.5t2-31.5q0-16-2-31.5t-7-29.5l86-65-39-68-99 42q-22-23-48.5-38.5T533-694l-13-106h-79l-14 106q-31 8-57.5 23.5T321-633l-99-41-39 68 86 64q-5 15-7 30t-2 32q0 16 2 31t7 30l-86 65 39 68 99-42q22 23 48.5 38.5T427-266l13 106Zm42-180q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm-2-140Z"
      /></svg
    >
  </button>

  <button
    onclick={() => (sessionState.sidebarOpen = !sessionState.sidebarOpen)}
    class="nav-icon vertical-layout-only"
    class:active={sessionState.sidebarOpen}
    title="Toggle Sidebar"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.25rem"
      viewBox="0 -960 960 960"
      width="1.25rem"
      fill="currentColor"
      ><path
        d="M160-160q-33 0-56.5-23.5T80-240v-480q0-33 23.5-56.5T160-800h640q33 0 56.5 23.5T880-720v480q0 33-23.5 56.5T800-160H160Zm540-453h100v-107H700v107Zm0 186h100v-106H700v106ZM160-240h460v-480H160v480Zm540 0h100v-107H700v107Z"
      /></svg
    >
  </button>
</nav>

<style>
  .navigation {
    width: 2.5rem;
    height: 100dvh;
    display: flex;
    flex-direction: column;
    background-color: color-mix(in srgb, var(--bg-surface), black 20%);
    box-shadow: 0 -1px 25px rgba(0, 0, 0, 0.1);
    align-items: center;
    z-index: 100;
  }

  .nav-icon {
    background: transparent;
    color: var(--text-ui-muted);
    border: none;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0.6rem;
    cursor: pointer;
    transition:
      background-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      color 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .nav-icon:hover,
  .nav-icon.active {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
  }

  .vertical-layout-only {
    display: none;
  }

  @media (max-width: 1024px) {
    .navigation {
      width: 100%;
      height: 2.5rem;
      flex-direction: row;
      justify-content: center;
      padding: 0;
      border-right: none;
      border-top: 0.0625rem solid var(--border);
    }

    .vertical-layout-only {
      display: flex;
    }
  }
</style>
