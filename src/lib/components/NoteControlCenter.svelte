<script lang="ts">
  /**
   * Control Center sidebar containing date, tags, and section shortcuts.
   */
  import type { NoteContentResponse, NoteSection } from '$lib/types/notes';
  import NoteDate from './NoteDate.svelte';
  import NoteSectionShortcuts from './NoteSectionShortcuts.svelte';
  import NoteTags from './NoteTags.svelte';

  let { noteContent, sections, onSelect } = $props<{
    noteContent: NoteContentResponse | null;
    sections: NoteSection[];
    onSelect: (name: string) => void;
  }>();
</script>

<div class="note-control-center">
  <div class="sidebar-section">
    <NoteDate {noteContent} />
  </div>

  <div class="sidebar-section">
    <h3 class="sidebar-title">Tags</h3>
    <NoteTags {noteContent} />
  </div>

  <div class="sidebar-section">
    <h3 class="sidebar-title">Sections</h3>
    <NoteSectionShortcuts {sections} {onSelect} />
  </div>
</div>

<style>
  .note-control-center {
    width: 260px;
    height: 100%;
    flex-shrink: 0;
    padding: 3rem 1.5rem;
    background-color: color-mix(in srgb, var(--bg-surface), black 10%);
    box-shadow: 0 1px 25px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .sidebar-section {
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
      width: 100%;
      height: auto;
      border-left: none;
      padding: 1.5rem;
      background-color: var(--bg-surface);
    }
  }
</style>
