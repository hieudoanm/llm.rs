import { fileURLToPath } from 'node:url';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [react()],
  resolve: {
    // Keep the `@/*` -> `src/*` alias from tsconfig.json working in tests.
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/__tests__/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      // App Router pages (`src/app/**`) are exercised end to end by Playwright
      // (`e2e/`); unit coverage targets the `src/lib` + `src/components`. The
      // multi-step wizard components (`src/components/jobs/**`) only render
      // through those pages, so they are covered by Playwright too.
      include: ['src/lib/**/*.{ts,tsx}', 'src/components/**/*.{ts,tsx}'],
      exclude: ['src/**/*.test.{ts,tsx}', 'src/components/jobs/**'],
      thresholds: {
        statements: 80,
        branches: 80,
        functions: 80,
        lines: 80,
      },
    },
  },
});
