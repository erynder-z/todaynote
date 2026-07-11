<script lang="ts">
  /**
   * Control Center sidebar containing date, tags, and thread shortcuts.
   */
  import { slide } from 'svelte/transition';
  import type { NoteContentResponse, NoteThread } from '$lib/interfaces/notes';
  import { t } from '$lib/utils/i18n';
  import { useShortcuts } from '$lib/utils/shortcuts';
  import { sessionState } from '../stores/sessionState.svelte';
  import NoteDate from './NoteDate.svelte';
  import NoteTags from './NoteTags.svelte';
  import NoteThreadShortcuts from './NoteThreadShortcuts.svelte';
  import ThreadShortcutsModeToggle from './ThreadShortcutsModeToggle.svelte';

  let {
    noteContent,
    threads,
    onSelect,
    width = 22, // rem
    isResizing = false,
  } = $props<{
    noteContent: NoteContentResponse | null;
    threads: NoteThread[];
    onSelect: (threadId: string) => void;
    width?: number;
    isResizing?: boolean;
  }>();

  const toggleSidebar = () => {
    sessionState.sidebarOpen = !sessionState.sidebarOpen;
  };

  const handleThreadSelect = (threadId: string) => {
    if (sessionState.threadShortcutsMode === 'navigation') {
      onSelect(threadId);
    } else {
      const selectedThread = threads.find(
        (t: { id: string }) => t.id === threadId,
      );
      if (selectedThread) {
        sessionState.selectedThreadForOptions = selectedThread;
        sessionState.activePopup = 'threadOptions';
      }
    }
  };

  useShortcuts({
    toggleSidebar: () => toggleSidebar(),
  });
</script>

<div
  class="sidebar"
  class:closed={!sessionState.sidebarOpen}
  class:resizing={isResizing}
  transition:slide={{ duration: 200, axis: 'x' }}
  style="width: {width}rem; --content-width: {width - 3}rem;"
>
  <button
    class="toggle-btn horizontal-only"
    onclick={toggleSidebar}
    title={$t('navigation.toggle_sidebar')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1rem"
      viewBox="0 -960 960 960"
      width="1rem"
      fill="currentColor"
      ><path d="M360-120v-720h80v720h-80Zm160-160v-400l200 200-200 200Z" /></svg
    >
  </button>

  <div class="sidebar-content">
    <div class="sidebar-sectio">
      <NoteDate {noteContent} />
    </div>

    <div class="sidebar-section">
      <h3 class="sidebar-title">{$t('search.tags')}</h3>
      <NoteTags {noteContent} />
    </div>

    <div class="sidebar-section">
      <div class="threads-title-container">
        <h3 class="sidebar-title">{$t('search.threads')}</h3>
        <ThreadShortcutsModeToggle />
      </div>
      <NoteThreadShortcuts {threads} onSelect={handleThreadSelect} />
    </div>
  </div>
</div>

<style>
  .sidebar {
    height: 100%;
    flex-shrink: 0;
    padding: 3rem 1.5rem;
    background-color: color-mix(in srgb, var(--bg-surface), black 10%);
    box-shadow: 0 1px 25px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    position: relative;
  }

  .sidebar-content {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: var(--content-width);
    flex-shrink: 0;
  }

  .toggle-btn {
    position: absolute;
    top: 50%;
    left: 0;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-ui-muted);
    padding: 0.75rem 0.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0 0.5rem 0.5rem 0;
    transition:
      padding 0.2s,
      background-color 0.2s,
      color 0.2s;
    z-index: 10;
  }

  .toggle-btn:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-color: var(--accent);
  }

  .sidebar-section {
    margin-top: 2rem;
  }

  .sidebar-title {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-ui-muted);
    margin-bottom: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }

  .threads-title-container {
    display: flex;
    justify-content: space-between;
  }

  @media (min-width: 1025px) {
    .sidebar {
      opacity: 1;
    }

    .sidebar.resizing {
      transition: none !important;
    }
  }

  @media (min-width: 1025px) {
    .sidebar {
      transition:
        width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        box-shadow 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      opacity: 1;
    }

    .sidebar.resizing {
      transition: none !important;
    }

    .sidebar.closed {
      width: 0 !important;
      padding-left: 0 !important;
      padding-right: 0 !important;
      opacity: 0;
      box-shadow: none;
      overflow: hidden;
      pointer-events: none;
    }
  }
</style>
