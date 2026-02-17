const extensions = require("./utils/extensions.cjs");

module.exports = [
  {
    files: extensions.all,
    rules: {
      "@typescript-eslint/array-type": [
        "error",
        { default: "generic", readonly: "generic" },
      ],
      "@typescript-eslint/consistent-type-definitions": ["error", "type"],
      "@typescript-eslint/consistent-type-imports": [
        "error",
        {
          prefer: "type-imports",
          disallowTypeAnnotations: true,
        },
      ],
      "func-style": ["error", "expression", { allowArrowFunctions: true }],
      curly: ["error", "all"],
      "prefer-const": "error",
      "no-var": "error",
      "no-sequences": "error",
      "no-multi-str": "error",
      "no-multi-assign": "error",
      "no-lone-blocks": "error",
      "no-lonely-if": "error",
      quotes: ["error", "single", { avoidEscape: true }],
    },
  },
];
