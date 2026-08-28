import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'jsdom',
    include: ['site/src/**/*.test.ts'],
    setupFiles: ['site/src/test-setup.ts'],
  },
});
