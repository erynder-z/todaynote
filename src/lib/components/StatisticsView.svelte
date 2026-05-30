<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { AppStatistics, InsightResponse } from '$lib/interfaces/notes';
  import { locale, t } from '../utils/i18n';

  let stats = $state<AppStatistics | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      stats = await invoke<AppStatistics>('get_statistics');
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const averageChars = $derived(
    stats && stats.totalNotes > 0
      ? Math.round(stats.totalCharacters / stats.totalNotes)
      : 0,
  );

  /**
   * Returns a localized weekday name for a given index (0 = Monday, 6 = Sunday).
   */
  const getDayName = (index: number, length: 'long' | 'short' = 'long') => {
    // 2024-01-01 was a Monday
    const date = new Date(Date.UTC(2024, 0, 1 + index));
    return new Intl.DateTimeFormat($locale, { weekday: length }).format(date);
  };

  const maxWeekdayValue = $derived(
    stats ? Math.max(...stats.weekdayDistribution, 1) : 1,
  );

  const formatInsight = (insight: InsightResponse) => {
    const params: Record<string, string | number> = { ...insight.params };
    if (params.day_index !== undefined) {
      params.day = getDayName(Number(params.day_index), 'long');
    }
    return $t(insight.key, params);
  };
</script>

<div class="statistics-container">
  {#if loading}
    <div class="loading">{$t('notes.loading')}</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if stats}
    <div class="hero-stats">
      <div class="streak-card">
        <div class="streak-info">
          <span class="streak-value">{stats.currentStreak}</span>
          <span class="streak-label">{$t('statistics.currentStreak')}</span>
        </div>
        <div class="streak-best">
          {$t('statistics.bestStreak')}: <strong>{stats.bestStreak}</strong>
        </div>
      </div>

      <div class="insights-card">
        <h3>{$t('statistics.insights')}</h3>
        <ul class="insights-list">
          {#each stats.insights as insight}
            <li>{formatInsight(insight)}</li>
          {/each}
          {#if stats.insights.length === 0}
            <li>{$t('statistics.insights.empty')}</li>
          {/if}
        </ul>
      </div>
    </div>
    <div class="stats-grid">
      <div class="stat-card">
        <span class="stat-value">{stats.totalNotes}</span>
        <span class="stat-label">{$t('statistics.totalNotes')}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{stats.totalTags}</span>
        <span class="stat-label">{$t('statistics.totalTags')}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{stats.totalThreads}</span>
        <span class="stat-label">{$t('statistics.totalThreads')}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{stats.totalCharacters.toLocaleString()}</span>
        <span class="stat-label">{$t('statistics.totalCharacters')}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{stats.totalWords.toLocaleString()}</span>
        <span class="stat-label">{$t('statistics.totalWords')}</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{averageChars}</span>
        <span class="stat-label">{$t('statistics.averageCharacters')}</span>
      </div>
    </div>

    <div class="distribution-section">
      <h3>{$t('statistics.weeklyDistribution')}</h3>
      <div class="bar-chart">
        {#each stats.weekdayDistribution as value, i}
          <div class="bar-wrapper">
            <div
              class="bar"
              style="height: {(value / maxWeekdayValue) * 100}%"
              title="{value} words"
            ></div>
            <span class="bar-label">{getDayName(i, 'short')}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="sections">
      <section class="top-tags">
        <h3>{$t('statistics.topTags')}</h3>
        <div class="tag-list">
          {#each stats.topTags as tag}
            <div class="tag-item">
              <span class="tag-name">#{tag.name}</span>
              <span class="tag-count">{tag.count}</span>
            </div>
          {/each}
          {#if stats.topTags.length === 0}
            <div class="empty-state">No tags found</div>
          {/if}
        </div>
      </section>

      <section class="top-threads">
        <h3>{$t('statistics.topThreads')}</h3>
        <div class="thread-list">
          {#each stats.topThreads as thread}
            <div class="thread-item">
              <span class="thread-name">{thread.name}</span>
              <span class="thread-count">{thread.count}</span>
            </div>
          {/each}
          {#if stats.topThreads.length === 0}
            <div class="empty-state">No threads found</div>
          {/if}
        </div>
      </section>

      <section class="recent-activity">
        <h3>{$t('statistics.recentActivity')}</h3>
        <div class="activity-list">
          {#each stats.dailyStats.slice(-7).reverse() as day}
            <div class="activity-item">
              <span class="activity-date">{day.date}</span>
              <span class="activity-chars"
                >{day.characterCount} {$t('statistics.chars')}</span
              >
            </div>
          {/each}
          {#if stats.dailyStats.length === 0}
            <div class="empty-state">No activity found</div>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .statistics-container {
    padding: 1.5rem;
    color: var(--text-main);
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .hero-stats {
    display: grid;
    grid-template-columns: 1fr 1.5fr;
    gap: 1.5rem;
  }

  @media (max-width: 768px) {
    .hero-stats {
      grid-template-columns: 1fr;
    }
  }

  .streak-card {
    background: linear-gradient(
      135deg,
      var(--accent),
      color-mix(in srgb, var(--accent), black 20%)
    );
    color: white;
    padding: 1.5rem;
    border-radius: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .streak-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    line-height: 1;
  }

  .streak-value {
    font-size: 3rem;
    font-weight: 900;
  }

  .streak-label {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.1rem;
    opacity: 0.9;
    margin-top: 0.5rem;
  }

  .streak-best {
    margin-top: 1rem;
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.2);
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
  }

  .insights-card {
    background-color: var(--bg-surface);
    padding: 1.5rem;
    border-radius: 1rem;
    border: 1px solid var(--border);
  }

  .insights-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .insights-list li {
    font-size: 0.95rem;
    line-height: 1.4;
    padding-left: 1.5rem;
    position: relative;
    color: var(--text-main);
  }

  .insights-list li::before {
    content: '✦';
    position: absolute;
    left: 0;
    color: var(--accent);
  }

  .distribution-section {
    background-color: var(--bg-surface);
    padding: 1.5rem;
    border-radius: 1rem;
    border: 1px solid var(--border);
  }

  .bar-chart {
    display: flex;
    height: 10rem;
    padding-top: 1rem;
    gap: 0.5rem;
  }

  .bar-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
    height: 100%;
    gap: 0.5rem;
  }

  .bar {
    width: 100%;
    max-width: 2.5rem;
    background-color: var(--accent);
    border-radius: 0.25rem 0.25rem 0 0;
    transition:
      height 0.3s ease,
      opacity 0.2s ease;
    opacity: 0.7;
  }

  .bar:hover {
    opacity: 1;
  }

  .bar-label {
    font-size: 0.7rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .loading,
  .error {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .error {
    color: var(--error);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    gap: 1rem;
  }

  .stat-card {
    background-color: var(--bg-surface);
    padding: 1rem;
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    border: 1px solid var(--border);
    transition: transform 0.2s ease;
  }

  .stat-card:hover {
    transform: translateY(-2px);
    border-color: var(--accent);
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 800;
    color: var(--accent);
  }

  .stat-label {
    font-size: 0.7rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05rem;
    margin-top: 0.25rem;
    text-align: center;
  }

  .sections {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
  }

  @media (max-width: 1024px) {
    .sections {
      grid-template-columns: 1fr;
    }
  }

  h3 {
    margin: 0 0 1rem 0;
    font-size: 0.9rem;
    font-weight: 700;
    color: var(--text-main);
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
    text-transform: uppercase;
    letter-spacing: 0.05rem;
  }

  .tag-list,
  .activity-list,
  .thread-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .tag-item,
  .activity-item,
  .thread-item {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background-color: var(--bg-surface);
    border-radius: 0.35rem;
    font-size: 0.9rem;
    border: 1px solid transparent;
  }

  .tag-item:hover,
  .activity-item:hover,
  .thread-item:hover {
    border-color: var(--border);
  }

  .tag-name,
  .thread-name {
    color: var(--accent);
    font-weight: 600;
  }

  .tag-count,
  .activity-chars,
  .thread-count {
    color: var(--text-muted);
    font-size: 0.8rem;
  }

  .activity-date {
    font-family: var(--font-mono);
    color: var(--text-main);
  }

  .empty-state {
    text-align: center;
    padding: 1rem;
    color: var(--text-muted);
    font-size: 0.85rem;
    font-style: italic;
  }
</style>
