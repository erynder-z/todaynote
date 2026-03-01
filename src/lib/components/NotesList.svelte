<script lang="ts">
  import { onMount } from 'svelte';
  import { settings, t } from '$lib';
  import type { FormattedNote } from '$lib/types/notes';
  import { listNotes } from '$lib/utils/folder';

  let notes: FormattedNote[] = $state([]);
  let isLoading = $state(true);

  onMount(() => {
    const unsubscribe = settings.subscribe(($settings) => {
      if ($settings.notes_folder) loadNotes();
    });
    return unsubscribe;
  });

  const loadNotes = async () => {
    isLoading = true;
    const loadedNotes = await listNotes();
    if (loadedNotes) notes = loadedNotes;
    isLoading = false;
  };
</script>

<div class="notes-section">
  {#if isLoading}
    <div class="status-msg">{$t('notes.loading')}</div>
  {:else if notes.length > 0}
    <div class="notes-list">
      {#each notes as note (note.filename)}
        <div class="note-item">
          <span class="note-name">{note.formatted_name}</span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="status-msg">{$t('notes.list.empty')}</div>
  {/if}
</div>

<style>
  .notes-section {
    margin: 1rem 0;
  }

  .notes-list {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .note-item {
    padding: 1rem 1.5rem;
    background-color: var(--bg-surface);
    border-radius: 10px;
    border: 1px solid var(--border);
    cursor: pointer;
    transition:
      transform 0.2s,
      border-color 0.2s;
    color: var(--text-main);
  }

  .note-item:hover {
    transform: translateX(5px);
    border-color: var(--accent);
  }

  .status-msg {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }
</style>
