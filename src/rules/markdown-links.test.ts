import { markdownLinksRule } from "@/rules/markdown-links.js";

describe("markdown_links rule", () => {
  test("flags standard markdown link [text](url)", () => {
    const result = markdownLinksRule.run(
      "Check [Google](https://google.com) for info.",
    );
    expect(result.length).toBeGreaterThan(0);
  });

  test("flags markdown link with title", () => {
    const result = markdownLinksRule.run(
      'Check [Google](https://google.com "Google Homepage") for info.',
    );
    expect(result.length).toBeGreaterThan(0);
  });

  test("does not flag bare URL", () => {
    const result = markdownLinksRule.run("Visit https://google.com for info.");
    expect(result).toEqual([]);
  });

  test("does not flag inside fenced code block", () => {
    const content = "```\n[link](https://example.com)\n```";
    const result = markdownLinksRule.run(content);
    expect(result).toEqual([]);
  });

  test("does not flag inside fenced code block with language", () => {
    const content = "```markdown\n[link](https://example.com)\n```";
    const result = markdownLinksRule.run(content);
    expect(result).toEqual([]);
  });

  test("does not flag inside inline code", () => {
    const result = markdownLinksRule.run("Use `[link](url)` syntax.");
    expect(result).toEqual([]);
  });

  test("does not flag image syntax ![alt](url)", () => {
    const result = markdownLinksRule.run(
      "![alt text](https://example.com/img.png)",
    );
    expect(result).toEqual([]);
  });

  test("reports multiple violations on different lines", () => {
    const content =
      "[one](https://one.com)\nplain text\n[two](https://two.com)";
    const result = markdownLinksRule.run(content);
    expect(result.length).toBeGreaterThanOrEqual(2);
  });

  test("does not flag reference-style definitions [label]: url", () => {
    const result = markdownLinksRule.run("[google]: https://google.com");
    expect(result).toEqual([]);
  });

  test("does not flag links inside good-example tags", () => {
    const content =
      "<good-example>\n[link](https://example.com)\n</good-example>";
    const result = markdownLinksRule.run(content);
    expect(result).toEqual([]);
  });

  test("does not flag links inside bad-example tags", () => {
    const content =
      "<bad-example>\n[link](https://example.com)\n</bad-example>";
    const result = markdownLinksRule.run(content);
    expect(result).toEqual([]);
  });

  test("does not flag links inside underscore example tags", () => {
    const content =
      "<good_example>\n[link](https://example.com)\n</good_example>";
    const result = markdownLinksRule.run(content);
    expect(result).toEqual([]);
  });

  test("flags links outside example tags", () => {
    const content =
      "<good-example>\n[inside](https://example.com)\n</good-example>\n[outside](https://example.com)";
    const result = markdownLinksRule.run(content);
    expect(result.length).toBe(1);
  });

  test("reports multiple violations on same line", () => {
    const content = "[one](https://one.com) and [two](https://two.com)";
    const result = markdownLinksRule.run(content);
    expect(result.length).toBeGreaterThanOrEqual(2);
  });

  test("does not leak code block state from example block", () => {
    const content =
      "<good-example>\n```\nsome code\n</good-example>\n[link](https://example.com)";
    const result = markdownLinksRule.run(content);
    expect(result.length).toBeGreaterThan(0);
  });

  test("returns empty for clean content", () => {
    const result = markdownLinksRule.run("Plain text with no links at all.");
    expect(result).toEqual([]);
  });

  test('has correct name "markdown_links"', () => {
    expect(markdownLinksRule.name).toBe("markdown_links");
  });

  test("has non-empty description", () => {
    expect(markdownLinksRule.description.length).toBeGreaterThan(0);
  });
});
