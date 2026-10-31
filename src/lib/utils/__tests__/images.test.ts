// Import vitest utilities first
import { beforeEach, describe, expect, it, vi } from "vitest";

// Mock @tauri-apps/api/core before any other imports that might use it
// Use vi.hoisted to ensure the mock variable is available before the mock is called
const { mockInvoke } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
	invoke: mockInvoke,
	convertFileSrc: vi.fn((path: string) => `asset://${path}`),
}));
vi.mock("../../stores/sessionState.svelte", () => ({
	sessionState: { isLinux: false },
}));

// Now import the module under test and other dependencies
import {
	buildImageFragment,
	fileUriToPath,
	hasSupportedImageExtension,
	localImagePathsFromClipboardText,
	parseImageFragment,
	stripImageFragment,
} from "../images.ts";

describe("image fragment utils", () => {
	beforeEach(() => {
		mockInvoke.mockReset();
	});

	describe("parseImageFragment", () => {
		it("parses size-only fragments", () => {
			expect(parseImageFragment("images/a.png#small")).toEqual({
				size: "small",
				align: null,
			});
		});

		it("parses align-only fragments", () => {
			expect(parseImageFragment("images/a.png#center")).toEqual({
				size: null,
				align: "center",
			});
		});

		it("parses combined size and align fragments", () => {
			expect(parseImageFragment("images/a.png#medium-right")).toEqual({
				size: "medium",
				align: "right",
			});
		});

		it("returns nulls without a fragment", () => {
			expect(parseImageFragment("images/a.png")).toEqual({
				size: null,
				align: null,
			});
		});

		it("returns nulls for unknown fragment values", () => {
			expect(parseImageFragment("images/a.png#huge")).toEqual({
				size: null,
				align: null,
			});
		});
	});

	describe("buildImageFragment", () => {
		it("omits defaults and builds combined fragments", () => {
			expect(buildImageFragment("medium", null)).toBe("#medium");
			expect(buildImageFragment("medium", "left")).toBe("#medium");
			expect(buildImageFragment("large", "center")).toBe("#large-center");
			expect(buildImageFragment(null, "right")).toBe("#right");
			expect(buildImageFragment(null, null)).toBe("");
		});
	});

	describe("stripImageFragment", () => {
		it("removes trailing fragments", () => {
			expect(stripImageFragment("images/a.png#medium-center")).toBe(
				"images/a.png",
			);
			expect(stripImageFragment("images/a.png#small")).toBe("images/a.png");
		});

		it("leaves plain paths untouched", () => {
			expect(stripImageFragment("images/a.png")).toBe("images/a.png");
			expect(stripImageFragment("https://example.com/a.png")).toBe(
				"https://example.com/a.png",
			);
		});
	});

	describe("fragment round trip", () => {
		it("preserves size and align through parse and build", () => {
			const src = "images/a.png";
			for (const size of ["small", "medium", "large"] as const) {
				for (const align of ["left", "center", "right"] as const) {
					const full = `${src}${buildImageFragment(size, align)}`;
					const parsed = parseImageFragment(full);
					expect(parsed).toEqual({
						size,
						align: align === "left" ? null : align,
					});
					expect(stripImageFragment(full)).toBe(src);
				}
			}
		});
	});
});

describe("fileUriToPath", () => {
	it("decodes local file URIs", () => {
		expect(fileUriToPath("file:///home/user/pic.png")).toBe(
			"/home/user/pic.png",
		);
		expect(
			fileUriToPath(
				"file:///home/stfnb/Pictures/Screenshots/Screenshot%20From%202026-05-11%2023-02-53.png",
			),
		).toBe(
			"/home/stfnb/Pictures/Screenshots/Screenshot From 2026-05-11 23-02-53.png",
		);
	});

	it("accepts localhost host", () => {
		expect(fileUriToPath("file://localhost/home/user/pic.png")).toBe(
			"/home/user/pic.png",
		);
	});

	it("strips the leading slash from Windows drive paths", () => {
		expect(fileUriToPath("file:///C:/Users/user/pic.png")).toBe(
			"C:/Users/user/pic.png",
		);
	});

	it("rejects non-file and remote URIs", () => {
		expect(fileUriToPath("https://example.com/pic.png")).toBeNull();
		expect(fileUriToPath("file://nas/share/pic.png")).toBeNull();
		expect(fileUriToPath("not a uri")).toBeNull();
	});
});

describe("hasSupportedImageExtension", () => {
	it("accepts allow-listed extensions case-insensitively", () => {
		expect(hasSupportedImageExtension("a.png")).toBe(true);
		expect(hasSupportedImageExtension("Screenshot.JPG")).toBe(true);
		expect(hasSupportedImageExtension("a.webp")).toBe(true);
	});

	it("rejects unsupported and missing extensions", () => {
		expect(hasSupportedImageExtension("a.svg")).toBe(false);
		expect(hasSupportedImageExtension("notes.txt")).toBe(false);
		expect(hasSupportedImageExtension("noext")).toBe(false);
	});
});

describe("localImagePathsFromClipboardText", () => {
	it("extracts image paths from a uri-list", () => {
		const uriList =
			"# comment\nfile:///home/user/a.png\nfile:///home/user/b.jpg\n";
		expect(localImagePathsFromClipboardText(uriList, "")).toEqual([
			"/home/user/a.png",
			"/home/user/b.jpg",
		]);
	});

	it("ignores remote URIs and non-image files", () => {
		const uriList =
			"https://example.com/a.png\nfile:///home/user/notes.txt\nfile:///home/user/c.webp";
		expect(localImagePathsFromClipboardText(uriList, "")).toEqual([
			"/home/user/c.webp",
		]);
	});

	it("falls back to plain text paths", () => {
		expect(localImagePathsFromClipboardText("", "/home/user/pic.png")).toEqual([
			"/home/user/pic.png",
		]);
		expect(
			localImagePathsFromClipboardText("", "file:///home/user/pic%20one.png"),
		).toEqual(["/home/user/pic one.png"]);
	});

	it("dedupes entries and ignores plain non-path text", () => {
		const uriList = "file:///home/user/a.png";
		expect(
			localImagePathsFromClipboardText(
				`${uriList}\n${uriList}`,
				"file:///home/user/a.png",
			),
		).toEqual(["/home/user/a.png"]);
		expect(localImagePathsFromClipboardText("", "just some text")).toEqual([]);
	});
});
