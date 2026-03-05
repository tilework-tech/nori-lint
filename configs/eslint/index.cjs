const eslintConfigPrettier = require("eslint-config-prettier");

const ignores = require("./ignores.cjs");
const importOrder = require("./importOrder.cjs");
const jsdoc = require("./jsdoc.cjs");
const recommended = require("./recommended/index.cjs");
const soot = require("./soot.cjs");
const extensions = require("./utils/extensions.cjs");

module.exports = {
  default: [
    ...recommended,
    ...importOrder,
    ...jsdoc,
    ...soot,
    ...ignores,

    {
      files: extensions.all,
      rules: {
        "react/react-in-jsx-scope": "off",
        "@typescript-eslint/ban-ts-comment": "off",
        "@typescript-eslint/ban-ts-ignore": "off",
        "@typescript-eslint/no-explicit-any": "off",
        "no-unused-vars": "off",
        "@typescript-eslint/no-unused-vars": [
          "warn",
          {
            argsIgnorePattern: "^_",
            varsIgnorePattern: "^_",
            caughtErrorsIgnorePattern: "^_",
          },
        ],
      },
    },

    {
      files: [
        "**/*.config.js",
        "**/*-config.js",
        "**/*.config.ts",
        "**/*-config.ts",
      ],
      rules: {
        "@typescript-eslint/no-var-requires": "off",
      },
    },

    eslintConfigPrettier,
  ],
};
