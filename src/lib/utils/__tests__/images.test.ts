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

// Now import the module under test and other dependencies
import {
	buildImageFragment,
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
