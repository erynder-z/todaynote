<script lang="ts">
  /**
   * Container component that renders the appropriate result component
   * based on search mode, layout, and selected tag.
   */
  import {
    NoteSearchResults,
    SearchMasonryLayout,
    settings,
    TagSearchResults,
    ThreadSearchResults,
    ThreadTagMasonryLayout,
  } from '$lib';
  import type {
    SearchResult,
    TagSearchResult,
    ThreadSearchResult,
  } from '$lib/interfaces/notes';

  import type { SearchResultsContainerProps } from '$lib/interfaces/ui';

  let {
    results,
    searchMode,
    selectedTag,
    query,
    nav,
    masonryLayout = $bindable(null),
    onSelectNote,
    onSelectThread,
    onSelectTag,
  }: SearchResultsContainerProps = $props();
</script>

<div class="results-scroll">
  {#if searchMode === 'notes' || selectedTag}
    {#if settings.notesListLayout === 'masonry'}
      <SearchMasonryLayout
        bind:this={masonryLayout}
        results={results as SearchResult[]}
        {nav}
        {query}
        onSelect={onSelectNote}
      />
    {:else}
      <NoteSearchResults
        results={results as SearchResult[]}
        {nav}
        {query}
        onSelect={onSelectNote}
      />
    {/if}
  {:else if searchMode === 'threads'}
    {#if settings.notesListLayout === 'masonry'}
      <ThreadTagMasonryLayout
        bind:this={masonryLayout}
        results={results as ThreadSearchResult[]}
        {nav}
        {searchMode}
        onSelect={onSelectThread}
      />
    {:else}
      <ThreadSearchResults
        results={results as ThreadSearchResult[]}
        {nav}
        onSelect={onSelectThread}
      />
    {/if}
  {:else if settings.notesListLayout === 'masonry'}
    <ThreadTagMasonryLayout
      bind:this={masonryLayout}
      results={results as TagSearchResult[]}
      {nav}
      {searchMode}
      onSelect={onSelectTag}
    />
  {:else}
    <TagSearchResults
      results={results as TagSearchResult[]}
      {nav}
      onSelect={onSelectTag}
    />
  {/if}
</div>

<style>
  .results-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }
</style>
