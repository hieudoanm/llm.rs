import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { loadApi, setupApiTests, teardownApiTests } from '../apiTestUtils';

beforeEach(() => {
  setupApiTests();
});

afterEach(() => {
  teardownApiTests();
});

describe('API_BASE', () => {
  it('defaults to the local backend', async () => {
    const { API_BASE } = await loadApi();
    expect(API_BASE).toBe('http://127.0.0.1:8000');
  });

  it('strips a trailing slash from the configured URL', async () => {
    const { API_BASE } = await loadApi('https://api.example.com/');
    expect(API_BASE).toBe('https://api.example.com');
  });
});
