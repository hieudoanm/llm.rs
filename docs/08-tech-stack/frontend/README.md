# llm — Frontend Best Practices

The **why** lives in [`08-tech-stack.md`](../../08-tech-stack.md). These documents are the **how**.

## Groups

| Document                                             | Covers                      |
| ---------------------------------------------------- | --------------------------- |
| [`core/nextjs.md`](./core/nextjs.md)                 | App Router, env, API client |
| [`core/typescript.md`](./core/typescript.md)         | Strict TS                   |
| [`core/tanstack-query.md`](./core/tanstack-query.md) | Polling jobs                |
| [`core/visx.md`](./core/visx.md)                     | Result charts               |
| [`styles/tailwindcss.md`](./styles/tailwindcss.md)   | Utilities                   |
| [`styles/daisyui.md`](./styles/daisyui.md)           | Components/theme            |
| [`testing/vitest.md`](./testing/vitest.md)           | Unit (Vitest, jsdom)        |
| [`testing/playwright.md`](./testing/playwright.md)   | E2E                         |
| [`../tools/eslint.md`](../tools/eslint.md)           | Lint                        |
| [`../tools/prettier.md`](../tools/prettier.md)       | Format                      |

## Shared rules

1. Arrow functions and named exports for components.
2. No `any`. Generate or hand-write types from OpenAPI.
3. DaisyUI first.
4. `'use client'` only where the component needs browser APIs (file input, query).
5. `NEXT_PUBLIC_API_URL` points at `http://127.0.0.1:8000` in Compose.
6. Never train or parse whole CSVs in the browser beyond a tiny preview.
7. ESLint lints; Prettier formats — ESLint never formats.

## Coding conventions

### File size limits

To maintain code readability and maintainability, source files should not exceed **200 lines**. A 10% margin of error (up to 220 lines) is acceptable for reasonable cases. When a file approaches this limit, consider splitting it into smaller, focused modules.

### File naming conventions

Two conventions, no exceptions:

| File type                                                              | Convention                      | Examples                                                                          |
| ---------------------------------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------- |
| **React components** (`.tsx` exporting a component)                    | **PascalCase**                  | `Button.tsx`, `ApiAlert.tsx`, `Navbar.tsx`, `ChartWrapper.tsx`, `ModelReport.tsx` |
| **Hooks** (`.ts` exporting a `use*` hook)                              | **camelCase with `use` prefix** | `useRlColumns.ts`, `useDebounce.ts`                                               |
| **Utilities / helpers / constants** (`.ts` exporting functions/values) | **camelCase**                   | `cn.ts`, `api.ts`, `metrics.ts`, `svgToPng.ts`, `apiTestUtils.ts`                 |
| **Test files**                                                         | **Match source exactly**        | `Button.tsx` → `Button.test.tsx`; `apiTestUtils.ts` → `apiTestUtils.test.ts`      |

Rationale:

- **PascalCase components** = React ecosystem standard; distinguishes components from HTML elements in JSX (`<Button />` vs `<button />`).
- **camelCase hooks/utils** = JavaScript/TypeScript standard for functions/variables; `use*` prefix is the hook convention.
- **Test matches source** = Predictable location; no cognitive overhead finding tests.

### Function declarations

All functions should use **arrow function syntax** consistently:

```typescript
// ✅ Correct
export const myFunction = (param: string): string => {
  return param.toUpperCase();
};

// ❌ Avoid
export function myFunction(param: string): string {
  return param.toUpperCase();
}
```

This convention applies to:

- Exported functions
- Component functions
- Helper functions
- Hook functions
- Test helper functions

The arrow function syntax provides better consistency with modern TypeScript/React patterns and works well with the functional programming style used throughout the codebase.
