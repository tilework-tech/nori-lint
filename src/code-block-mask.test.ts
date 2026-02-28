import { maskCodeBlocks, restoreCodeBlocks } from "@/code-block-mask.js";

describe("maskCodeBlocks", () => {
  test("content without code blocks passes through unchanged", () => {
    const input = "just some text\nwith multiple lines\n";
    const { masked, blocks } = maskCodeBlocks(input);

    expect(masked).toBe(input);
    expect(blocks).toHaveLength(0);
  });

  test("code block content is not visible in masked output", () => {
    const input = "before\n```bash\necho hello\n```\nafter";
    const { masked } = maskCodeBlocks(input);

    expect(masked).not.toContain("echo hello");
    expect(masked).toContain("before");
    expect(masked).toContain("after");
  });

  test("multiple code blocks are all hidden from masked output", () => {
    const input =
      "intro\n```bash\necho 1\n```\nmiddle\n```typescript\nconst x = 1;\n```\nend";
    const { masked } = maskCodeBlocks(input);

    expect(masked).not.toContain("echo 1");
    expect(masked).not.toContain("const x = 1");
    expect(masked).toContain("intro");
    expect(masked).toContain("middle");
    expect(masked).toContain("end");
  });
});

describe("restoreCodeBlocks", () => {
  test("round-trips single code block", () => {
    const input = "before\n```bash\necho hello\n```\nafter";
    const { masked, blocks } = maskCodeBlocks(input);
    const restored = restoreCodeBlocks(masked, blocks);

    expect(restored).toBe(input);
  });

  test("round-trips multiple code blocks", () => {
    const input =
      "intro\n```bash\necho 1\n```\nmiddle\n```typescript\nconst x = 1;\n```\nend";
    const { masked, blocks } = maskCodeBlocks(input);
    const restored = restoreCodeBlocks(masked, blocks);

    expect(restored).toBe(input);
  });

  test("round-trips code blocks with language identifiers", () => {
    const input = "text\n```python\nprint('hi')\n```\nmore";
    const { masked, blocks } = maskCodeBlocks(input);
    const restored = restoreCodeBlocks(masked, blocks);

    expect(restored).toBe(input);
  });

  test("round-trips empty code blocks", () => {
    const input = "text\n```\n```\nmore";
    const { masked, blocks } = maskCodeBlocks(input);
    const restored = restoreCodeBlocks(masked, blocks);

    expect(restored).toBe(input);
  });

  test("round-trips code blocks preserving indentation", () => {
    const input = "text\n```bash\n  indented\n    more indented\n```\nmore";
    const { masked, blocks } = maskCodeBlocks(input);
    const restored = restoreCodeBlocks(masked, blocks);

    expect(restored).toBe(input);
  });

  test("preserves modifications to surrounding content", () => {
    const input = "some **bold** text\n```bash\necho hello\n```\nmore text";
    const { masked, blocks } = maskCodeBlocks(input);

    // Simulate LLM fixing bold text but leaving placeholders intact
    const modified = masked.replace("**bold**", "bold");
    const restored = restoreCodeBlocks(modified, blocks);

    expect(restored).toContain("bold");
    expect(restored).not.toContain("**bold**");
    expect(restored).toContain("```bash\necho hello\n```");
  });

  test("throws if a placeholder was removed from the content", () => {
    const input = "before\n```bash\necho 1\n```\nafter";
    const { masked: _masked, blocks } = maskCodeBlocks(input);

    // Remove the placeholder line — keep only original surrounding text
    const withoutPlaceholder = "before\nafter";

    expect(() => restoreCodeBlocks(withoutPlaceholder, blocks)).toThrow();
  });
});
