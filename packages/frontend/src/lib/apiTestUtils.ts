import { vi } from 'vitest';

export const fetchMock = vi.fn();
const originalApiUrl = process.env.NEXT_PUBLIC_API_URL;

export const jsonResponse = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });

export const textResponse = (body: string, status: number) =>
  new Response(body, { status, headers: { 'content-type': 'text/plain' } });

export const callAt = (index = 0) => fetchMock.mock.calls[index] as [string, RequestInit];

export const loadApi = async (base?: string) => {
  vi.resetModules();
  if (base === undefined) {
    delete process.env.NEXT_PUBLIC_API_URL;
  } else {
    process.env.NEXT_PUBLIC_API_URL = base;
  }
  return import('./api');
};

export const setupApiTests = () => {
  fetchMock.mockReset();
  vi.stubGlobal('fetch', fetchMock);
};

export const teardownApiTests = () => {
  vi.unstubAllGlobals();
  if (originalApiUrl === undefined) {
    delete process.env.NEXT_PUBLIC_API_URL;
  } else {
    process.env.NEXT_PUBLIC_API_URL = originalApiUrl;
  }
};
