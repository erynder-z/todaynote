<script lang="ts">
  import { appState, t } from '$lib';
  import type { PopupType } from '$lib/types/ui';

  const togglePopup = (type: PopupType) => {
    appState.update((state) => ({
      ...state,
      activePopup: state.activePopup === type ? null : type,
    }));
  };
</script>

<nav class="navigation">
  <button
    onclick={() => togglePopup('notesList')}
    class="nav-icon"
    title={$t('navigation.home')}
    aria-label={$t('navigation.home')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="fit-content"
      viewBox="0 -960 960 960"
      width="fit-content"
      fill="#e3e3e3"
      ><path
        d="M80-160v-160h160v160H80Zm240 0v-160h560v160H320ZM80-400v-160h160v160H80Zm240 0v-160h560v160H320ZM80-640v-160h160v160H80Zm240 0v-160h560v160H320Z"
      /></svg
    >
  </button>

  <button
    onclick={() => togglePopup('search')}
    class="nav-icon"
    title={$t('notes.search')}
    aria-label={$t('notes.search')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="fit-content"
      viewBox="0 -960 960 960"
      width="fit-content"
      fill="#e3e3e3"
      ><path
        d="M784-120 532-372q-30 24-69 38t-83 14q-109 0-184.5-75.5T120-580q0-109 75.5-184.5T380-840q109 0 184.5 75.5T640-580q0 44-14 83t-38 69l252 252-56 56ZM380-400q75 0 127.5-52.5T560-580q0-75-52.5-127.5T380-760q-75 0-127.5 52.5T200-580q0 75 52.5 127.5T380-400Z"
      /></svg
    >
  </button>

  <button
    onclick={() => togglePopup('folderSelector')}
    class="nav-icon"
    title={$t('navigation.settings')}
    aria-label={$t('navigation.settings')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="fit-content"
      viewBox="0 -960 960 960"
      width="fit-content"
      fill="#e3e3e3"
      ><path
        d="m370-80-16-128q-13-5-24.5-12T307-235l-119 50L78-375l103-78q-1-7-1-13.5v-27q0-6.5 1-13.5L78-585l110-190 119 50q11-8 23-15t24-12l16-128h220l16 128q13 5 24.5 12t22.5 15l119-50 110 190-103 78q1 7 1 13.5v27q0 6.5-2 13.5l103 78-110 190-118-50q-11 8-23 15t-24 12L590-80H370Zm70-80h79l14-106q31-8 57.5-23.5T639-327l99 41 39-68-86-65q5-14 7-29.5t2-31.5q0-16-2-31.5t-7-29.5l86-65-39-68-99 42q-22-23-48.5-38.5T533-694l-13-106h-79l-14 106q-31 8-57.5 23.5T321-633l-99-41-39 68 86 64q-5 15-7 30t-2 32q0 16 2 31t7 30l-86 65 39 68 99-42q22 23 48.5 38.5T427-266l13 106Zm42-180q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm-2-140Z"
      /></svg
    >
  </button>
</nav>

<style>
  .navigation {
    position: fixed;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.8rem;
    z-index: 100;
  }

  .nav-icon {
    width: 3rem;
    height: 3rem;
    border-radius: 50%;
    background-color: #646cff;
    color: white;
    border: none;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0.75rem;
    cursor: pointer;
    font-size: 1.2rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    transition:
      transform 0.2s,
      background-color 0.2s;
  }

  .nav-icon:hover {
    background-color: #535bf2;
    transform: scale(1.1);
  }

  .nav-icon:active {
    transform: scale(0.95);
  }

  @media (prefers-color-scheme: dark) {
    .nav-icon {
      background-color: #747bff;
    }

    .nav-icon:hover {
      background-color: #646cff;
    }
  }
</style>
