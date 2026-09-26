// Import vitest utilities first
import { beforeEach, describe, expect, it, vi } from "vitest";

// Mock @tauri-apps/api/core before any other imports that might use it
// Use vi.hoisted to ensure the mock variable is available before the mock is called
const { mockInvoke } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
	invoke: mockInvoke,
}));

// Now import the module under test and other dependencies
import type { SearchResult } from "$lib/interfaces/notes";
import {
	filterResultsByFilename,
	filterResultsByScore,
	limitResults,
	processSearchResults,
	processSearchResultsClient,
	sortSearchResults,
} from "../search";

// Test fixtures
const mockResults: SearchResult[] = [
	{
		filename: "2024-01-15_test.md",
		formattedName: "Jan 15, 2024",
		excerpt: "This is a test note",
		lineNumber: 1,
		score: 95,
		indices: [0, 4],
	},
	{
		filename: "2024-01-10_another.md",
		formattedName: "Jan 10, 2024",
		excerpt: "Another note here",
		lineNumber: 1,
		score: 80,
		indices: [0, 7],
	},
	{
		filename: "2024-01-20_recent.md",
		formattedName: "Jan 20, 2024",
		excerpt: "Most recent note",
		lineNumber: 1,
		score: 90,
		indices: [0, 4],
	},
	{
		filename: "2024-01-05_old.md",
		formattedName: "Jan 5, 2024",
		excerpt: "Old note",
		lineNumber: 1,
		score: 60,
		indices: [0, 2],
	},
];

describe("filterResultsByScore", () => {
	it("filters results by minimum score", () => {
		const result = filterResultsByScore(mockResults, 80);
		expect(result).toHaveLength(3);
		expect(result.every((r) => r.score >= 80)).toBe(true);
	});

	it("returns empty array when no results match", () => {
		const result = filterResultsByScore(mockResults, 100);
		expect(result).toHaveLength(0);
	});

	it("returns all results when minScore is 0", () => {
		const result = filterResultsByScore(mockResults, 0);
		expect(result).toHaveLength(4);
	});

	it("handles empty input array", () => {
		const result = filterResultsByScore([], 50);
		expect(result).toHaveLength(0);
	});
});

describe("limitResults", () => {
	it("limits results to specified count", () => {
		const result = limitResults(mockResults, 2);
		expect(result).toHaveLength(2);
		expect(result).toEqual(mockResults.slice(0, 2));
	});

	it("returns all results when limit is greater than array length", () => {
		const result = limitResults(mockResults, 10);
		expect(result).toHaveLength(4);
	});

	it("returns empty array when limit is 0", () => {
		const result = limitResults(mockResults, 0);
		expect(result).toHaveLength(0);
	});

	it("handles empty input array", () => {
		const result = limitResults([], 2);
		expect(result).toHaveLength(0);
	});
});

describe("filterResultsByFilename", () => {
	it("filters results by filename (case-insensitive)", () => {
		const result = filterResultsByFilename(mockResults, "test");
		expect(result).toHaveLength(1);
		expect(result[0].filename).toBe("2024-01-15_test.md");
	});

	it("matches partial filename", () => {
		const result = filterResultsByFilename(mockResults, "another");
		expect(result).toHaveLength(1);
		expect(result[0].filename).toBe("2024-01-10_another.md");
	});

	it("handles case-insensitive matching", () => {
		const result = filterResultsByFilename(mockResults, "TEST");
		expect(result).toHaveLength(1);
		expect(result[0].filename).toBe("2024-01-15_test.md");
	});

	it("returns all results when filter is empty", () => {
		const result = filterResultsByFilename(mockResults, "");
		expect(result).toHaveLength(4);
	});

	it("returns empty array when no matches found", () => {
		const result = filterResultsByFilename(mockResults, "nonexistent");
		expect(result).toHaveLength(0);
	});

	it("handles empty input array", () => {
		const result = filterResultsByFilename([], "test");
		expect(result).toHaveLength(0);
	});
});

describe("sortSearchResults", () => {
	it("sorts by score in descending order by default", () => {
		const result = sortSearchResults([...mockResults]);
		expect(result[0].score).toBe(95);
		expect(result[1].score).toBe(90);
		expect(result[2].score).toBe(80);
		expect(result[3].score).toBe(60);
	});

	it("sorts by filename in ascending order", () => {
		const result = sortSearchResults([...mockResults], "filename");
		expect(result[0].filename).toBe("2024-01-05_old.md");
		expect(result[1].filename).toBe("2024-01-10_another.md");
		expect(result[2].filename).toBe("2024-01-15_test.md");
		expect(result[3].filename).toBe("2024-01-20_recent.md");
	});

	it("sorts by date in descending order", () => {
		const result = sortSearchResults([...mockResults], "date");
		// Dates are extracted from filename: 2024-01-20, 2024-01-15, 2024-01-10, 2024-01-05
		expect(result[0].filename).toBe("2024-01-20_recent.md");
		expect(result[1].filename).toBe("2024-01-15_test.md");
		expect(result[2].filename).toBe("2024-01-10_another.md");
		expect(result[3].filename).toBe("2024-01-05_old.md");
	});

	it("handles empty input array", () => {
		const result = sortSearchResults([], "score");
		expect(result).toHaveLength(0);
	});

	it("does not mutate original array", () => {
		const original = [...mockResults];
		sortSearchResults(mockResults, "score");
		expect(mockResults).toEqual(original);
	});
});

describe("processSearchResultsClient", () => {
	it("applies all filters and sorting correctly", () => {
		const result = processSearchResultsClient(mockResults, {
			minScore: 80,
			filenameFilter: "2024-01",
			sortBy: "score",
			maxResults: 2,
		});

		expect(result).toHaveLength(2);
		// Should be filtered by score >= 80, then limited to 2
		expect(result.every((r) => r.score >= 80)).toBe(true);
		expect(result[0].score).toBeGreaterThanOrEqual(result[1].score);
	});

	it("applies only minScore filter", () => {
		const result = processSearchResultsClient(mockResults, {
			minScore: 85,
		});

		expect(result).toHaveLength(2);
		expect(result.every((r) => r.score >= 85)).toBe(true);
	});

	it("applies only maxResults filter", () => {
		const result = processSearchResultsClient(mockResults, {
			maxResults: 1,
		});

		expect(result).toHaveLength(1);
	});

	it("applies only filenameFilter", () => {
		const result = processSearchResultsClient(mockResults, {
			filenameFilter: "test",
		});

		expect(result).toHaveLength(1);
		expect(result[0].filename).toBe("2024-01-15_test.md");
	});

	it("applies only sortBy", () => {
		const result = processSearchResultsClient(mockResults, {
			sortBy: "filename",
		});

		expect(result[0].filename).toBe("2024-01-05_old.md");
	});

	it("handles empty input array", () => {
		const result = processSearchResultsClient([], {
			minScore: 50,
			maxResults: 5,
			filenameFilter: "test",
			sortBy: "score",
		});

		expect(result).toHaveLength(0);
	});

	it("handles empty options", () => {
		const result = processSearchResultsClient(mockResults, {});

		expect(result).toHaveLength(4);
	});

	it("handles undefined options", () => {
		const result = processSearchResultsClient(mockResults);

		expect(result).toHaveLength(4);
	});

	it("combines filters correctly - score and filename", () => {
		const result = processSearchResultsClient(mockResults, {
			minScore: 80,
			filenameFilter: "test",
		});

		// Only the test file has score >= 80
		expect(result).toHaveLength(1);
		expect(result[0].filename).toBe("2024-01-15_test.md");
	});

	it("handles minScore of 0", () => {
		const result = processSearchResultsClient(mockResults, {
			minScore: 0,
			maxResults: 2,
		});

		expect(result).toHaveLength(2);
	});
});

describe("processSearchResults", () => {
	beforeEach(() => {
		mockInvoke.mockReset();
	});

	it("calls backend with correct parameters", async () => {
		mockInvoke.mockResolvedValue(mockResults);

		const options = {
			minScore: 50,
			maxResults: 10,
			filenameFilter: "test",
			sortBy: "score",
		};

		await processSearchResults(mockResults, options);

		expect(mockInvoke).toHaveBeenCalledWith("process_search_results", {
			results: mockResults,
			minScore: 50,
			maxResults: 10,
			filenameFilter: "test",
			sortBy: "score",
		});
	});

	it("returns backend results on success", async () => {
		const backendResults: SearchResult[] = [
			{ ...mockResults[0], score: 99 },
			{ ...mockResults[1], score: 85 },
		];
		mockInvoke.mockResolvedValue(backendResults);

		const result = await processSearchResults(mockResults, { minScore: 80 });

		expect(result).toEqual(backendResults);
	});

	it("falls back to original results when backend throws error", async () => {
		mockInvoke.mockRejectedValue(new Error("Backend error"));

		// Suppress console.error for this test since we're testing the fallback behavior
		const consoleErrorSpy = vi
			.spyOn(console, "error")
			.mockImplementation(() => {});

		const result = await processSearchResults(mockResults, { minScore: 80 });

		expect(result).toEqual(mockResults);

		consoleErrorSpy.mockRestore();
	});

	it("uses default sortBy value of 'score' when not provided", async () => {
		mockInvoke.mockResolvedValue([]);

		const options = {
			minScore: 50,
			maxResults: 10,
			filenameFilter: "test",
			// sortBy intentionally omitted
		};

		await processSearchResults(mockResults, options);

		expect(mockInvoke).toHaveBeenCalledWith("process_search_results", {
			results: mockResults,
			minScore: 50,
			maxResults: 10,
			filenameFilter: "test",
			sortBy: "score",
		});
	});

	it("handles empty options object", async () => {
		mockInvoke.mockResolvedValue(mockResults);

		await processSearchResults(mockResults);

		expect(mockInvoke).toHaveBeenCalledWith("process_search_results", {
			results: mockResults,
			minScore: undefined,
			maxResults: undefined,
			filenameFilter: undefined,
			sortBy: "score",
		});
	});

	it("handles empty results array", async () => {
		mockInvoke.mockResolvedValue([]);

		const result = await processSearchResults([], { minScore: 50 });

		expect(mockInvoke).toHaveBeenCalledWith("process_search_results", {
			results: [],
			minScore: 50,
			maxResults: undefined,
			filenameFilter: undefined,
			sortBy: "score",
		});
		expect(result).toEqual([]);
	});
});
