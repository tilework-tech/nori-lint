const importPlugin = require("eslint-plugin-import");

const extensions = require("./utils/extensions.cjs");

module.exports = [
  {
    files: extensions.all,
    plugins: {
      import: importPlugin,
    },
    rules: {
      "import/order": [
        "error",
        {
          groups: ["builtin", "external", "internal", "type"],
          "newlines-between": "always",
          alphabetize: {
            order: "asc",
            caseInsensitive: true,
          },
        },
      ],
    },
    settings: {
      "import/external-module-folders": ["node_modules"],
      "import/resolver": {
        typescript: {},
      },
    },
  },
];
