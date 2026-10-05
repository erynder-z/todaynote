<script lang="ts">
  /**
   * Component for displaying aggregated content (threads) from multiple notes.
   */
  import { sessionState, settings, t, toast } from '$lib';
  import type { AggregatedThreadItem } from '$lib/interfaces/notes';
  import { locale } from '$lib/utils/i18n';
  import { notesService } from '$lib/utils/notes';
  import { aggregationItemPlugin } from '../plugins/aggregationItemPlugin';
  import { linkOpenerPlugin } from '../plugins/linkOpenerPlugin';
  import MilkdownEditor from './MilkdownEditor.svelte';

  let aggregation = $derived(sessionState.aggregatedThread);

  const editorPlugins = $derived([linkOpenerPlugin, ...aggregationItemPlugin]);

  const createInternalLink = (filename: string, threadId: string) =>
    `https://todaynote.internal/open/${encodeURIComponent(filename)}/${encodeURIComponent(threadId)}`;

  const renderItem = (item: AggregatedThreadItem) => {
    const formattedDate = notesService.formatNoteDateFromMetadata(
      item.created,
      item.filename,
      $locale,
      settings.dateFormatStyle,
    );

    const link = createInternalLink(item.filename, item.threadId);

    return `## [${formattedDate}](${link})

${item.content}`;
  };

  let combinedContent = $derived.by(() => {
    if (!aggregation || aggregation.items.length === 0) return '';
    const item_separator = '\n\n---\n\n';

    return aggregation.items.map(renderItem).join(item_separator);
  });

  /**
   * Opens the original note
   */
  const openOriginalNote = async (filename: string, threadId: string) => {
    if (!settings.notesFolder) return;
    const path = `${settings.notesFolder}/${filename}`;
    const content = await notesService.readNoteContent(path);
    if (content !== null) {
      sessionState.todayNotePath = path;
      sessionState.todayNoteContent = content;
      sessionState.pendingThreadJump = threadId;
      sessionState.activePopup = null;
    } else {
      toast.error($t('notes.error.load'));
    }
  };

  // Handle container clicks directly to bypass ProseMirror's event loop (especially in readonly mode)
  const handleContainerClick = (event: MouseEvent) => {
    const anchor = (
      event.target as HTMLElement | null
    )?.closest<HTMLAnchorElement>('a[href]');
    if (!anchor) return;

    const rawHref = anchor.getAttribute('href');
    if (!rawHref?.includes('todaynote.internal/open/')) return;

    event.preventDefault();
    event.stopPropagation();

    const url = new URL(rawHref);

    if (url.protocol !== 'https:' || url.hostname !== 'todaynote.internal')
      return;

    const [, action, filename, threadId] = url.pathname.split('/');

    if (action !== 'open' || !filename) return;

    event.preventDefault();
    event.stopPropagation();

    openOriginalNote(
      decodeURIComponent(filename),
      decodeURIComponent(threadId ?? ''),
    );
  };
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="aggregation-container" onclickcapture={handleContainerClick}>
  {#if aggregation && aggregation.items.length > 0}
    <div class="editor-wrapper">
      <MilkdownEditor
        content={combinedContent}
        readonly
        plugins={editorPlugins}
      />
    </div>
  {:else}
    <div class="empty-state">
      <p class="muted">No content found for this thread.</p>
    </div>
  {/if}
</div>

<style>
  .aggregation-container {
    padding: 0 1rem;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-wrapper {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 1.5rem;
  }

  /* Override default Milkdown min-height and padding for the aggregation view */
  .editor-wrapper :global(.milkdown) {
    min-height: auto !important;
    height: 100%;
  }

  .editor-wrapper :global(.milkdown .editor) {
    padding-bottom: 2rem;
  }

  /* Styling for aggregation list items */
  .editor-wrapper :global(.milkdown .aggregation-list-item) {
    margin-top: 1.5rem;
    margin-bottom: 1.5rem;
    border-left: 0.25rem solid
      color-mix(in srgb, var(--accent), transparent 80%);
    padding: 0 1rem;
  }

  .editor-wrapper :global(.milkdown .aggregation-list-item:first-child) {
    margin-top: 0;
  }

  .editor-wrapper :global(.milkdown .aggregation-list-item h2:first-child) {
    display: flex;
    justify-content: flex-end;
    align-items: center;
  }

  .editor-wrapper :global(.milkdown .aggregation-list-item h2 a) {
    background: color-mix(in srgb, var(--accent), transparent 90%);
    border: 1px solid color-mix(in srgb, var(--accent), transparent 80%);
    color: var(--accent);
    padding: 0.3rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.9rem;
    font-weight: 600;
    text-decoration: none;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .editor-wrapper :global(.milkdown .aggregation-list-item h2 a:hover) {
    background: var(--accent);
    color: var(--accent-text);
    border-color: var(--accent);
  }

  .editor-wrapper
    :global(.milkdown .aggregation-list-item > :not(h2):first-child) {
    margin-top: 0.5rem;
  }

  /* Differentiate the horizontal rule separators */
  .editor-wrapper :global(.milkdown hr) {
    margin: 2rem 0;
    border-top: 1px dashed var(--border);
  }

  .empty-state {
    display: flex;
    justify-content: center;
    align-items: center;
    flex: 1;
    color: var(--text-muted);
    font-style: italic;
  }

  .muted {
    opacity: 0.7;
  }
</style>
