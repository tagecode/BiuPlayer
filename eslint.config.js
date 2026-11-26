import js from "@eslint/js";
import tseslint from "typescript-eslint";

const tsFiles = ["src/**/*.{ts,tsx}", "e2e/**/*.ts", "vite.config.ts"];

export default [
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src-tauri/**",
      "test-results/**",
      "playwright-report/**",
      ".tmp-build/**",
      ".tmp-test/**",
      ".native-e2e/**",
    ],
  },
  { ...js.configs.recommended, files: ["**/*.{js,mjs,cjs}"] },
  {
    files: ["scripts/**/*.{js,mjs}"],
    languageOptions: {
      globals: {
        console: "readonly",
        process: "readonly",
        fetch: "readonly",
        AbortSignal: "readonly",
        Buffer: "readonly",
        URL: "readonly",
      },
    },
  },
  ...tseslint.configs.recommended.map((config) => ({ ...config, files: tsFiles })),
  {
    files: tsFiles,
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  },
];
