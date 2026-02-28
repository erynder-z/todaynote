<script lang="ts">
  import { onMount } from 'svelte';
  import { settings, t } from '$lib';
  import type { FormattedNote } from '$lib/types/notes';
  import { listNotes } from '$lib/utils/folder';

  let notes: FormattedNote[] = $state([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(() => {
    const unsubscribe = settings.subscribe(($settings) => {
      if ($settings.notes_folder) {
        loadNotes();
      } else {
        notes = [];
        isLoading = false;
      }
    });

    return unsubscribe;
  });

  const loadNotes = async () => {
    try {
      isLoading = true;
      error = null;

      if (!$settings.notes_folder) {
        isLoading = false;
        return;
      }

      const loadedNotes = await listNotes();
      if (loadedNotes) notes = loadedNotes;
    } catch (err) {
      console.error('Error loading notes:', err);
      error = $t('notes.error.load');
    } finally {
      isLoading = false;
    }
  };
</script>

<div class="notes-section">
  {#if isLoading && $settings.notes_folder}
    <div class="loading">{$t('notes.loading')}</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if $settings.notes_folder && notes.length > 0}
    <div class="notes-list">
      {#each notes as note (note.filename)}
        <div class="note-item">
          <span class="note-name">{note.formatted_name}</span>
        </div>
      {/each}
    </div>
  {:else if $settings.notes_folder}
    <div class="empty-notes">{$t('notes.list.empty')}</div>
  {/if}
</div>

<style>
  .notes-section {
    margin: 1rem auto;
    max-width: 800px;
    width: 100%;
    padding: 0 1rem;
  }

  .notes-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 600px;
    margin: 0 auto;
    width: 100%;
  }

  .note-item {
    padding: 1rem;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    transition:
      transform 0.2s,
      box-shadow 0.2s;
    cursor: pointer;
    text-align: left;
  }

  .note-item:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  }

  .note-name {
    font-size: 1rem;
    font-weight: 500;
    color: #333;
  }

  .loading,
  .error,
  .empty-notes {
    padding: 1rem;
    border-radius: 8px;
    margin: 1rem auto 0;
    max-width: 500px;
    text-align: center;
  }

  .loading {
    color: #666;
  }

  .error {
    background-color: #ffebee;
    color: #c62828;
    border: 1px solid #ef9a9a;
  }

  .empty-notes {
    color: #999;
  }

  @media (prefers-color-scheme: dark) {
    .note-item {
      background-color: #3a3a3a;
      color: #f6f6f6;
    }

    .note-name {
      color: #f6f6f6;
    }

    .error {
      background-color: #3a2a2a;
      color: #ffb7b7;
      border: 1px solid #5a3a3a;
    }
  }
</style>
