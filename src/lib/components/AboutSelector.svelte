<script lang="ts">
  import { t } from '$lib/utils/i18n';
  /**
   * Button that opens the About popup.
   */
  import { sessionState } from '../stores/sessionState.svelte';

  const openAbout = () => {
    sessionState.activePopup = 'about';
  };
</script>

<button class="about-selector" onclick={openAbout}>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    height="1.25rem"
    viewBox="0 -960 960 960"
    width="1.25rem"
    fill="currentColor"
    ><path
      d="M440-280h80v-240h-80v240Zm68.5-331.5Q520-623 520-640t-11.5-28.5Q497-680 480-680t-28.5 11.5Q440-657 440-640t11.5 28.5Q463-600 480-600t28.5-11.5ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
    /></svg
  >
  {$t('about.title')}
</button>

<style>
  .about-selector {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.4rem;
    padding: 0.5rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      opacity 0.15s cubic-bezier(0.2, 0, 0, 1),
      transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
    color: var(--accent-text);
    background-color: var(--accent);
    min-width: 15ch;
  }

  .about-selector:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .about-selector:active {
    transform: translateY(0);
  }
</style>
