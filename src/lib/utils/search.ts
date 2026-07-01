/**
 * Search-related utilities and backend API calls.
 */

import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "$lib/interfaces/notes";

/**
 * Processes search results using backend filtering and sorting.
 */
export const processSearchResults = async (
	results: SearchResult[],
	options: {
		minScore?: number;
		maxResults?: number;
		filenameFilter?: string;
		sortBy?: string;
	} = {},
): Promise<SearchResult[]> => {
	try {
		return await invoke("process_search_results", {
			results,
			minScore: options.minScore,
			maxResults: options.maxResults,
			filenameFilter: options.filenameFilter,
			sortBy: options.sortBy || "score",
		});
	} catch (error) {
		console.error("Error processing search results:", error);
		// Fallback: return original results if backend call fails
		return results;
	}
};

/**
 * Filters search results by minimum score.
 */
export const filterResultsByScore = (
	results: SearchResult[],
	minScore: number,
): SearchResult[] => {
	return results.filter((result) => result.score >= minScore);
};

/**
 * Limits the number of search results.
 */
export const limitResults = (
	results: SearchResult[],
	maxResults: number,
): SearchResult[] => {
	return results.slice(0, maxResults);
};

/**
 * Filters search results by filename (case-insensitive).
 */
export const filterResultsByFilename = (
	results: SearchResult[],
	filenameFilter: string,
): SearchResult[] => {
	const filterLower = filenameFilter.toLowerCase();
	return results.filter((result) =>
		result.filename.toLowerCase().includes(filterLower),
	);
};

/**
 * Sorts search results by different criteria.
 */
export const sortSearchResults = (
	results: SearchResult[],
	sortBy: string = "score",
): SearchResult[] => {
	const sorted = [...results];

	switch (sortBy) {
		case "filename":
			sorted.sort((a, b) => a.filename.localeCompare(b.filename));
			break;
		case "date":
			// Sort by filename (which contains date) in reverse chronological order
			sorted.sort((a, b) => {
				const aDate = a.filename.split("_")[0] || "";
				const bDate = b.filename.split("_")[0] || "";
				return bDate.localeCompare(aDate); // Newest first
			});
			break;
		case "score":
		default:
			sorted.sort((a, b) => b.score - a.score);
	}

	return sorted;
};

/**
 * Client-side processing of search results (fallback when backend is unavailable).
 */
export const processSearchResultsClient = (
	results: SearchResult[],
	options: {
		minScore?: number;
		maxResults?: number;
		filenameFilter?: string;
		sortBy?: string;
	} = {},
): SearchResult[] => {
	let processed = [...results];

	// Filters
	if (options.minScore !== undefined)
		processed = filterResultsByScore(processed, options.minScore);

	if (options.filenameFilter)
		processed = filterResultsByFilename(processed, options.filenameFilter);

	// Sorting
	if (options.sortBy) processed = sortSearchResults(processed, options.sortBy);

	// Limits
	if (options.maxResults !== undefined)
		processed = limitResults(processed, options.maxResults);

	return processed;
};
