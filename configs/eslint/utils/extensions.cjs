const typescript = ["**/*.ts", "**/*.tsx", "**/*.mts", "**/*.cts"];
const javascript = ["**/*.js", "**/*.jsx", "**/*.mjs", "*.cjs"];

module.exports = {
  typescript,
  javascript,
  all: [...typescript, ...javascript],
};
