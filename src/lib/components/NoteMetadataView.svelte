<script lang="ts">
  /**
   * Displays the metadata of the currently loaded note
   * (filename, path, date, tags, threads and raw frontmatter).
   */
  import { t } from '$lib/utils/i18n';
  import { sessionState } from '../stores/sessionState.svelte';

  const note = $derived(sessionState.todayNoteContent);
  const filename = $derived(
    sessionState.todayNotePath?.split(/[\\/]/).pop() ?? '-',
  );
  const tags = $derived(note?.metadata.tags ?? []);
  const rawEntries = $derived(Object.entries(note?.metadata.raw ?? {}));
</script>

{#if note}
  <div class="metadata-view">
    <dl>
      <div class="metadata-row">
        <dt>{$t('note.metadata.filename')}</dt>
        <dd class="selectable">{filename}</dd>
      </div>
      <div class="metadata-row">
        <dt>{$t('note.metadata.path')}</dt>
        <dd class="selectable">{note.path || sessionState.todayNotePath}</dd>
      </div>
      <div class="metadata-row">
        <dt>{$t('note.metadata.date')}</dt>
        <dd class="selectable">{note.metadata.formattedDate}</dd>
      </div>
    </dl>
    {#if rawEntries.length > 0}
      <dl>
        {#each rawEntries as [key, value]}
          <div class="metadata-row">
            <dt class="selectable">{key}</dt>
            <dd class="selectable">{value}</dd>
          </div>
        {/each}
      </dl>
    {/if}
  </div>
{:else}
  <div class="empty-view">
    <p>{$t('note.metadata.empty')}</p>
  </div>
{/if}

<style>
  .metadata-view {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .metadata-row {
    display: flex;
    gap: 1rem;
    padding: 0.35rem 0;
    border-bottom: 1px solid color-mix(in srgb, var(--border), transparent 60%);
  }

  dt {
    flex-shrink: 0;
    width: 6.5rem;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    padding-top: 0.15rem;
  }

  dd {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-main);
    word-break: break-all;
    min-width: 0;
  }

  .selectable {
    user-select: text;
  }

  .empty-view {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    color: var(--text-muted);
    font-style: italic;
  }
</style>
