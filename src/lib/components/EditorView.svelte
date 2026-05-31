<script lang="ts">
  /**
   * View component that combines the note editor with the note sidebar.
   */

  import { fade } from 'svelte/transition';
  import type { NoteContentResponse } from '$lib/interfaces/notes';
  import { settings, t } from '../index';
  import { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import NoteEditor from './NoteEditor.svelte';
  import Sidebar from './Sidebar.svelte';

  let { noteContent = $bindable(), notePath } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
  }>();

  const editor = new EditorStore();

  /**
   * Indicates whether the sidebar is currently being resized.
   */
  let isResizing = $state(false);
  let windowWidth = $state(
    typeof window !== 'undefined' ? window.innerWidth : 1200,
  );
  let isVertical = $derived(windowWidth <= 1024);

  $effect(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * Automatically close/open sidebar based on window width to handle responsive layout changes.
   * Also persists manual changes when in desktop layout.
   */
  $effect(() => {
    // This effect handles the responsive transitions
    if (isVertical) {
      sessionState.sidebarOpen = false;
    } else {
      // Restore the user's desktop preference when returning to normal layout
      sessionState.sidebarOpen = settings.sidebarOpen;
    }
  });

  $effect(() => {
    // This effect persists manual toggles in horizontal layout
    const currentStatus = sessionState.sidebarOpen;
    if (!isVertical) settings.setSidebarOpen(currentStatus);
  });

  /**
   * Handles navigation to a specific thread, creating it if it doesn't exist.
   */
  const handleJump = async (name: string) => {
    // Update content to ensure thread exists
    if (!editor.threads.some((s) => s.name === name)) {
      const updatedContent = await editor.ensureThreadExists(name);
      noteContent = updatedContent;
    }

    if (editor.jumpToThread) editor.jumpToThread(name);

    // Close sidebar in vertical layout after jumping
    if (window.innerWidth <= 1024) sessionState.sidebarOpen = false;
  };

  /**
   * Initiates the resizing process for the sidebar.
   */
  const startResizing = () => {
    isResizing = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  };

  /**
   * Concludes the resizing process and persists the new width.
   */
  const stopResizing = () => {
    if (!isResizing) return;
    isResizing = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    settings.setControlCenterWidth(settings.controlCenterWidth);
  };

  /**
   * Updates the sidebar width based on mouse movement during resizing.
   */
  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing) return;

    const fontSize = parseFloat(
      getComputedStyle(document.documentElement).fontSize,
    );
    const minWidthRem = 15;
    const maxWidthRem = 40;
    const newWidthRem = (window.innerWidth - e.clientX) / fontSize;

    if (newWidthRem >= minWidthRem && newWidthRem <= maxWidthRem) {
      settings.controlCenterWidth = newWidthRem;
    }
  };
</script>

<svelte:window
  bind:innerWidth={windowWidth}
  onmousemove={handleMouseMove}
  onmouseup={stopResizing}
/>

<div class="note-container">
  {#if !sessionState.sidebarOpen}
    <button
      transition:fade={{ duration: 200 }}
      class="sidebar-open-btn desktop-only"
      onclick={() => (sessionState.sidebarOpen = true)}
      title={$t('navigation.toggle_sidebar')}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1rem"
        viewBox="0 -960 960 960"
        width="1rem"
        fill="currentColor"
        ><path
          d="M440-280v-400L240-480l200 200Zm80 160h80v-720h-80v720Z"
        /></svg
      >
    </button>
  {/if}

  <div class="note-layout">
    <div class="editor-main">
      <NoteEditor bind:noteContent {notePath} {editor} />
    </div>

    <!-- Resizer Handle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resizer"
      class:resizing={isResizing}
      class:visible={sessionState.sidebarOpen}
      onmousedown={startResizing}
    ></div>

    {#if sessionState.sidebarOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="vertical-layout"
        onclick={() => (sessionState.sidebarOpen = false)}
      ></div>
    {/if}

    <div class="sidebar-wrapper" class:open={sessionState.sidebarOpen}>
      <Sidebar
        {noteContent}
        threads={editor.threads}
        onSelect={handleJump}
        width={settings.controlCenterWidth}
      />
    </div>
  </div>
</div>

<style>
  .note-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-base);
    position: relative;
  }

  .sidebar-open-btn {
    position: absolute;
    top: 50%;
    right: 0;
    transform: translateY(-50%);
    z-index: 100;
    background: none;
    border: none;
    color: var(--text-ui-muted);
    padding: 0.75rem 0.25rem;
    border-radius: 0.5rem 0 0 0.5rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: -2px 0 8px rgba(0, 0, 0, 0.1);
    transition:
      padding 0.2s,
      background-color 0.2s,
      color 0.2s;
  }

  .sidebar-open-btn:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-color: var(--accent);
  }

  .note-layout {
    display: flex;
    flex: 1;
    gap: 0;
    min-height: 0;
    position: relative;
  }

  .editor-main {
    flex: 1;
    min-width: 0;
    padding: 3rem;
    overflow-y: auto;
    background-color: var(--bg-base);
  }

  .sidebar-wrapper {
    display: contents;
  }

  .resizer {
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background-color: transparent;
    transition:
      background-color 0.2s,
      opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 10;
    margin-left: -2px;
    margin-right: -2px;
    opacity: 0;
    pointer-events: none;
  }

  .resizer.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .resizer:hover,
  .resizer.resizing {
    background-color: var(--accent);
  }

  .vertical-layout {
    display: none;
  }

  @media (max-width: 1024px) {
    .editor-main {
      padding: 1.5rem;
    }

    .resizer,
    .resizer.visible {
      display: none;
    }

    .sidebar-wrapper {
      display: block;
      position: absolute;
      top: 0;
      right: 0;
      height: 100%;
      z-index: 1000;
      transform: translateX(100%);
      transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
      will-change: transform;
    }

    .sidebar-wrapper.open {
      display: block;
      transform: translateX(0);
    }

    .vertical-layout {
      display: block;
      position: absolute;
      inset: 0;
      background: rgba(0, 0, 0, 0.4);
      backdrop-filter: blur(2px);
      z-index: 999;
    }

    .desktop-only {
      display: none;
    }
  }
</style>
