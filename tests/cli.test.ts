import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { run } from "@/cli.js";

const makeTempDir = (): string => {
  return fs.mkdtempSync(path.join(os.tmpdir(), "nori-lint-test-"));
};

const withArgs = async (
  args: Array<string>,
): Promise<{ code: number; stdout: string; stderr: string }> => {
  const originalArgv = process.argv;
  const originalStdoutWrite = process.stdout.write;
  const originalStderrWrite = process.stderr.write;

  let stdout = "";
  let stderr = "";

  process.argv = ["node", "nori-lint", ...args];
  process.stdout.write = ((chunk: string) => {
    stdout += chunk;
    return true;
  }) as typeof process.stdout.write;
  process.stderr.write = ((chunk: string) => {
    stderr += chunk;
    return true;
  }) as typeof process.stderr.write;

  try {
    const code = await run();
    return { code, stdout, stderr };
  } finally {
    process.argv = originalArgv;
    process.stdout.write = originalStdoutWrite;
    process.stderr.write = originalStderrWrite;
  }
};

const smallSkillContent = () =>
  "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSome content.\n</required>\n";

const largeSkillContent = () =>
  Array.from({ length: 200 }, (_, i) => `line ${i + 1}`).join("\n");

const skillWithBoldText = () =>
  "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSome **bold** content.\n</required>\n";

const skillWithoutRequiredTags = () =>
  "---\nname: Test Skill\ndescription: A test skill\n---\n\nSome content without required tags.\n";

const skillWithUnclosedTag = () =>
  "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSome content.\n";

const skillWithRedundantTitle = () =>
  "---\nname: Test Skill\ndescription: A test skill\n---\n\n# The Test Skill\n\n<required>\nSome content.\n</required>\n";

const skillWithMarkdownLink = () =>
  "---\nname: Test Skill\ndescription: A test skill\n---\n\n<required>\nSee [the docs](https://example.com) for details.\n</required>\n";

const validConfigWithDisabledRules = (disabled: Array<string>) =>
  JSON.stringify({ anthropic_api_key: "sk-ant-fake-key", rules: { disabled } });

const validConfigWithEnabledRules = (enabled: Array<string>) =>
  JSON.stringify({ anthropic_api_key: "sk-ant-fake-key", rules: { enabled } });

let tempDirs: Array<string> = [];

const createTempDir = (): string => {
  const dir = makeTempDir();
  tempDirs.push(dir);
  return dir;
};

afterEach(() => {
  for (const dir of tempDirs) {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  tempDirs = [];
});

describe("CLI integration tests", () => {
  describe("basic behavior", () => {
    test("exits success when no skill files found", async () => {
      const dir = createTempDir();
      const { code } = await withArgs([dir]);
      expect(code).toBe(0);
    });

    test("exits success for valid skill file", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      const { code } = await withArgs([dir]);
      expect(code).toBe(0);
    });

    test("exits failure for skill file exceeding line limit", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), largeSkillContent());

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });

    test("reports only violating files when mixed", async () => {
      const dir = createTempDir();

      const goodDir = path.join(dir, "good-skill");
      fs.mkdirSync(goodDir);
      fs.writeFileSync(path.join(goodDir, "SKILL.md"), smallSkillContent());

      const badDir = path.join(dir, "bad-skill");
      fs.mkdirSync(badDir);
      fs.writeFileSync(path.join(badDir, "SKILL.md"), largeSkillContent());

      const { code, stdout } = await withArgs(["--format", "json", dir]);
      expect(code).toBe(1);

      const diagnostics = JSON.parse(stdout) as Array<{ file: string }>;
      const files = diagnostics.map((d) => d.file);
      expect(files.some((f) => f.includes("bad-skill"))).toBe(true);
      expect(files.some((f) => f.includes("good-skill"))).toBe(false);
    });

    test("discovers skill files in nested directories", async () => {
      const dir = createTempDir();
      const nested = path.join(dir, "a", "b", "nested-skill");
      fs.mkdirSync(nested, { recursive: true });
      fs.writeFileSync(path.join(nested, "SKILL.md"), largeSkillContent());

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });
  });

  describe("directory path argument", () => {
    test("accepts directory path argument", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      const { code } = await withArgs([dir]);
      expect(code).toBe(0);
    });

    test("accepts directory path with violations", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), largeSkillContent());

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });

    test("reports error for nonexistent directory", async () => {
      const { code, stderr } = await withArgs(["/tmp/nonexistent-dir-xyz-123"]);
      expect(code).toBe(1);
      expect(stderr).toContain("does not exist");
    });

    test("reports error for file instead of directory", async () => {
      const dir = createTempDir();
      const filePath = path.join(dir, "somefile.txt");
      fs.writeFileSync(filePath, "hello");

      const { code, stderr } = await withArgs([filePath]);
      expect(code).toBe(1);
      expect(stderr).toContain("is not a directory");
    });
  });

  describe("--format text", () => {
    test("format text produces output with rule name in brackets", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), largeSkillContent());

      const { code, stdout } = await withArgs(["--format", "text", dir]);
      expect(code).toBe(1);
      expect(stdout).toContain("[line_count]");
    });
  });

  describe("--format json", () => {
    test("format json outputs valid JSON with violation", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), largeSkillContent());

      const { code, stdout } = await withArgs(["--format", "json", dir]);
      expect(code).toBe(1);

      const diagnostics = JSON.parse(stdout) as Array<{ rule: string }>;
      expect(Array.isArray(diagnostics)).toBe(true);
      expect(diagnostics.some((d) => d.rule === "line_count")).toBe(true);
    });

    test("format json outputs empty array when no violations", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      const { stdout } = await withArgs(["--format", "json", dir]);
      const diagnostics = JSON.parse(stdout) as Array<unknown>;
      expect(diagnostics).toEqual([]);
    });

    test("format json outputs multiple diagnostics across files", async () => {
      const dir = createTempDir();

      const skillDir1 = path.join(dir, "skill-one");
      fs.mkdirSync(skillDir1);
      fs.writeFileSync(path.join(skillDir1, "SKILL.md"), largeSkillContent());

      const skillDir2 = path.join(dir, "skill-two");
      fs.mkdirSync(skillDir2);
      fs.writeFileSync(
        path.join(skillDir2, "SKILL.md"),
        skillWithoutRequiredTags(),
      );

      const { stdout } = await withArgs(["--format", "json", dir]);
      const diagnostics = JSON.parse(stdout) as Array<{
        rule: string;
        file: string;
      }>;
      expect(diagnostics.length).toBeGreaterThanOrEqual(2);

      const files = new Set(diagnostics.map((d) => d.file));
      expect(files.size).toBeGreaterThanOrEqual(2);
    });

    test("format invalid value prints error", async () => {
      const dir = createTempDir();
      const { code, stderr } = await withArgs(["--format", "xml", dir]);
      expect(code).toBe(1);
      expect(stderr.toLowerCase()).toContain("format");
    });
  });

  describe("--help", () => {
    test("help flag prints usage and exits success", async () => {
      const { code, stdout } = await withArgs(["--help"]);
      expect(code).toBe(0);
      expect(stdout).toContain("Usage");
      expect(stdout).toContain("nori-lint");
    });
  });

  describe("required_tags rule", () => {
    test("exits failure for skill file missing required tags", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(
        path.join(skillDir, "SKILL.md"),
        skillWithoutRequiredTags(),
      );

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });

    test("format json includes required_tags violation", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(
        path.join(skillDir, "SKILL.md"),
        skillWithoutRequiredTags(),
      );

      const { stdout } = await withArgs(["--format", "json", dir]);
      const diagnostics = JSON.parse(stdout) as Array<{ rule: string }>;
      expect(diagnostics.some((d) => d.rule === "required_tags")).toBe(true);
    });
  });

  describe("unclosed_tags rule", () => {
    test("exits failure for skill file with unclosed tag", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), skillWithUnclosedTag());

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });
  });

  describe("--config", () => {
    test("without config, deterministic rules still run", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), largeSkillContent());

      const { code, stderr } = await withArgs([dir]);
      expect(code).toBe(1);
      expect(stderr.toLowerCase()).toContain("skipping llm rules");
    });

    test("config flag with nonexistent file prints error", async () => {
      const dir = createTempDir();
      const { code, stderr } = await withArgs([
        "--config",
        "/tmp/nonexistent-config-xyz.json",
        dir,
      ]);
      expect(code).toBe(1);
      expect(stderr.length).toBeGreaterThan(0);
    });

    test("config flag with invalid JSON prints error", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "bad-config.json");
      fs.writeFileSync(configPath, "not valid json {{{");

      const { code, stderr } = await withArgs(["--config", configPath, dir]);
      expect(code).toBe(1);
      expect(stderr.length).toBeGreaterThan(0);
    });

    test("config flag with missing api key prints error", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(configPath, JSON.stringify({ rules: {} }));

      const { code, stderr } = await withArgs(["--config", configPath, dir]);
      expect(code).toBe(1);
      expect(stderr.toLowerCase()).toContain("anthropic_api_key");
    });
  });

  describe("redundant_title rule", () => {
    test("exits failure for skill file with redundant title heading", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(
        path.join(skillDir, "SKILL.md"),
        skillWithRedundantTitle(),
      );

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });
  });

  describe("rules config (enable/disable)", () => {
    test("disabled rule does not produce diagnostics", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(
        configPath,
        validConfigWithDisabledRules(["bold_italics"]),
      );

      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), skillWithBoldText());

      const { stdout } = await withArgs([
        "--config",
        configPath,
        "--format",
        "json",
        dir,
      ]);
      const diagnostics = JSON.parse(stdout) as Array<{ rule: string }>;
      expect(diagnostics.some((d) => d.rule === "bold_italics")).toBe(false);
    });

    test("non-disabled rules still fire", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(
        configPath,
        validConfigWithDisabledRules(["bold_italics"]),
      );

      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(
        path.join(skillDir, "SKILL.md"),
        skillWithoutRequiredTags(),
      );

      const { stdout } = await withArgs([
        "--config",
        configPath,
        "--format",
        "json",
        dir,
      ]);
      const diagnostics = JSON.parse(stdout) as Array<{ rule: string }>;
      expect(diagnostics.some((d) => d.rule === "required_tags")).toBe(true);
    });

    test("enabled rules only runs specified rules", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(
        configPath,
        validConfigWithEnabledRules(["required_tags"]),
      );

      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      // This file has bold text AND missing required tags
      const content =
        "---\nname: Test Skill\ndescription: A test skill\n---\n\nSome **bold** content without required tags.\n";
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), content);

      const { stdout } = await withArgs([
        "--config",
        configPath,
        "--format",
        "json",
        dir,
      ]);
      const diagnostics = JSON.parse(stdout) as Array<{ rule: string }>;
      expect(diagnostics.some((d) => d.rule === "required_tags")).toBe(true);
      expect(diagnostics.some((d) => d.rule === "bold_italics")).toBe(false);
    });

    test("config with both enabled and disabled prints error", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(
        configPath,
        JSON.stringify({
          anthropic_api_key: "sk-ant-fake-key",
          rules: { enabled: ["line_count"], disabled: ["bold_italics"] },
        }),
      );

      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      const { code, stderr } = await withArgs(["--config", configPath, dir]);
      expect(code).toBe(1);
      expect(stderr.length).toBeGreaterThan(0);
    });
  });

  describe("bold_italics rule", () => {
    test("exits failure for skill file with bold text", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), skillWithBoldText());

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });

    test("format json includes bold_italics violation with line number", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), skillWithBoldText());

      const { stdout } = await withArgs(["--format", "json", dir]);
      const diagnostics = JSON.parse(stdout) as Array<{
        rule: string;
        line: number | null;
      }>;
      const boldViolation = diagnostics.find((d) => d.rule === "bold_italics");
      expect(boldViolation).toBeDefined();
      expect(boldViolation!.line).toBeGreaterThan(0);
    });
  });

  describe("markdown_links rule", () => {
    test("exits failure for skill file with markdown link syntax", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(
        path.join(skillDir, "SKILL.md"),
        skillWithMarkdownLink(),
      );

      const { code } = await withArgs([dir]);
      expect(code).toBe(1);
    });
  });

  describe(".nori-lint.json auto-discovery", () => {
    test(".nori-lint.json in cwd is auto-discovered", async () => {
      const dir = createTempDir();
      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      // Write invalid config to prove it was loaded (should cause error)
      fs.writeFileSync(path.join(dir, ".nori-lint.json"), "not valid json {{{");

      const originalCwd = process.cwd();
      try {
        process.chdir(dir);
        const { code, stderr } = await withArgs([dir]);
        expect(code).toBe(1);
        expect(stderr.length).toBeGreaterThan(0);
      } finally {
        process.chdir(originalCwd);
      }
    });
  });

  describe("unknown rule warnings", () => {
    test("unknown rule in disabled list emits warning on stderr", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(
        configPath,
        validConfigWithDisabledRules(["nonexistent_rule"]),
      );

      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      const { stderr } = await withArgs(["--config", configPath, dir]);
      expect(stderr).toContain("nonexistent_rule");
    });

    test("unknown rule in enabled list emits warning on stderr", async () => {
      const dir = createTempDir();
      const configPath = path.join(dir, "config.json");
      fs.writeFileSync(
        configPath,
        validConfigWithEnabledRules(["nonexistent_rule"]),
      );

      const skillDir = path.join(dir, "my-skill");
      fs.mkdirSync(skillDir);
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), smallSkillContent());

      const { stderr } = await withArgs(["--config", configPath, dir]);
      expect(stderr).toContain("nonexistent_rule");
    });
  });
});
