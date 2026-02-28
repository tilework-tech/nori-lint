const PLACEHOLDER_PREFIX = "<!-- NORI_CODE_BLOCK_";
const PLACEHOLDER_SUFFIX = " -->";

export function maskCodeBlocks(content: string): {
  masked: string;
  blocks: Array<string>;
} {
  const blocks: Array<string> = [];
  const lines = content.split("\n");
  const result: Array<string> = [];
  let inCodeBlock = false;
  let currentBlock: Array<string> = [];

  for (const line of lines) {
    if (line.trim().startsWith("```")) {
      if (!inCodeBlock) {
        inCodeBlock = true;
        currentBlock = [line];
      } else {
        currentBlock.push(line);
        const blockIndex = blocks.length;
        blocks.push(currentBlock.join("\n"));
        result.push(`${PLACEHOLDER_PREFIX}${blockIndex}${PLACEHOLDER_SUFFIX}`);
        currentBlock = [];
        inCodeBlock = false;
      }
    } else if (inCodeBlock) {
      currentBlock.push(line);
    } else {
      result.push(line);
    }
  }

  // If we ended inside an unclosed code block, put the lines back as-is
  if (inCodeBlock) {
    for (const line of currentBlock) {
      result.push(line);
    }
  }

  return { masked: result.join("\n"), blocks };
}

export function restoreCodeBlocks(
  masked: string,
  blocks: Array<string>,
): string {
  let result = masked;

  for (let i = 0; i < blocks.length; i++) {
    const placeholder = `${PLACEHOLDER_PREFIX}${i}${PLACEHOLDER_SUFFIX}`;
    if (!result.includes(placeholder)) {
      throw new Error(`Code block placeholder ${i} was removed during LLM fix`);
    }
    result = result.replace(placeholder, blocks[i]);
  }

  return result;
}
