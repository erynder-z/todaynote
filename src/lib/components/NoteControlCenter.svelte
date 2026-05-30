<script lang="ts">
  /**
   * Control Center sidebar containing date, tags, and thread shortcuts.
   */
  import { t } from '$lib/utils/i18n';
  import type { NoteContentResponse, NoteThread } from '$lib/interfaces/notes';
  import NoteDate from './NoteDate.svelte';
  import NoteTags from './NoteTags.svelte';
  import NoteThreadShortcuts from './NoteThreadShortcuts.svelte';

  let {
    noteContent,
    threads,
    onSelect,
    width = 22, // rem
  } = $props<{
    noteContent: NoteContentResponse | null;
    threads: NoteThread[];
    onSelect: (name: string) => void;
    width?: number;
  }>();
</script>

<div class="note-control-center" style="width: {width}rem">
  <div class="sidebar-thread">
    <NoteDate {noteContent} />
  </div>

  <div class="sidebar-thread">
    <h3 class="sidebar-title">{$t('search.tags')}</h3>
    <NoteTags {noteContent} />
  </div>

  <div class="sidebar-thread">
    <h3 class="sidebar-title">{$t('search.threads')}</h3>
    <NoteThreadShortcuts {threads} {onSelect} />
  </div>
</div>

<style>
  .note-control-center {
    height: 100%;
    flex-shrink: 0;
    padding: 3rem 1.5rem;
    background-color: color-mix(in srgb, var(--bg-surface), black 10%);
    box-shadow: 0 1px 25px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .sidebar-thread {
    margin-bottom: 2.5rem;
  }

  .sidebar-title {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-ui-muted);
    margin-bottom: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }

  @media (max-width: 1024px) {
    .note-control-center {
      width: 100% !important;
      height: auto;
      border-left: none;
      padding: 1.5rem;
      background-color: var(--bg-surface);
    }
  }
</style>
