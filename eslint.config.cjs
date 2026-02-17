const configs = require("./configs/index.cjs");

module.exports = [
  {
    ignores: ["dist/", "node_modules/", ".git/", "**/build/", "**/.worktrees/"],
  },
  ...configs.eslint,

  {
    files: ["configs/**/*.cjs"],
    rules: {
      "@typescript-eslint/no-var-requires": "off",
    },
  },

  {
    files: ["**/*.cjs"],
    rules: {
      "@typescript-eslint/no-var-requires": "off",
    },
  },

  {
    files: ["**/*.ts"],
    rules: {
      "func-style": "off",
      "jsdoc/require-jsdoc": "off",
      "jsdoc/require-returns": "off",
      "jsdoc/require-param-description": "off",
    },
  },
];
