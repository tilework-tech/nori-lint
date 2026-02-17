import { lineCountRule } from "@/rules/line-count.js";

describe("line_count rule", () => {
  test("returns empty for short file", () => {
    const content = "Line one\nLine two\nLine three";
    const result = lineCountRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty at exactly 150 lines", () => {
    const content = Array.from({ length: 150 }, (_, i) => `Line ${i + 1}`).join(
      "\n",
    );
    const result = lineCountRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation at 151 lines", () => {
    const content = Array.from({ length: 151 }, (_, i) => `Line ${i + 1}`).join(
      "\n",
    );
    const result = lineCountRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("151");
    expect(result[0].message).toContain("150");
    expect(result[0].line).toBeUndefined();
    expect(result[0].snippet).toBeUndefined();
  });

  test('has correct name "line_count"', () => {
    expect(lineCountRule.name).toBe("line_count");
  });

  test("has non-empty description", () => {
    expect(lineCountRule.description.length).toBeGreaterThan(0);
  });
});
