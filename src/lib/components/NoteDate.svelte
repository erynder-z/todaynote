<script lang="ts">
  /**
   * Component for displaying a note's formatted date.
   * Extracts and formats the localized date from the provided note content.
   */
  import type { NoteDateProps } from '#lib/interfaces/notes.ts';
  import { locale } from '#lib/utils/i18n.ts';
  import { notesService } from '#lib/utils/notes.ts';
  import { settings } from '../stores/settings.svelte.ts';

  let { noteContent }: NoteDateProps = $props();

  let date = $derived(
    noteContent?.path
      ? notesService.formatNoteDateFromMetadata(
          noteContent.metadata.raw['created'],
          noteContent.path.split(/[/\\]/).pop() || '',
          $locale,
          settings.dateFormatStyle,
        )
      : '',
  );
</script>

{#if date}
  <div class="date">{date}</div>
{/if}

<style>
  .date {
    font-size: 1.1rem;
    color: var(--text-main);
    font-weight: 700;
    letter-spacing: -0.01em;
    text-align: center;
    width: 100%;
  }
</style>
