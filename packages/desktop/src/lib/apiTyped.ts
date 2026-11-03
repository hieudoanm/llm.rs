/**
 * Typed API client generated from the committed OpenAPI contract (FEATURE-18).
 *
 * The type surface comes from `src/lib/apiTypes.ts`, which is produced from
 * `docs/06-api-specification/backend/openapi.yaml` by
 * `pnpm run openapi:types`. Regenerate it whenever the backend routes or
 * schemas change — the backend contract test fails otherwise.
 *
 * The runtime behaviour (timeout, retries, `ApiError`) comes from
 * `src/lib/api.ts`; the typed helpers call through that wrapper, so a typed
 * call has the same resilience and error taxonomy as an untyped one.
 */

import { apiGet, apiPatch, apiPost, apiRequest, type ApiRequestOptions } from '@/lib/api';
import type { paths } from '@/lib/apiTypes';

/** A JSON body of the `POST`/`PATCH` operation at `Path`. */
type JsonBody<Path extends keyof paths, Method extends 'post' | 'patch'> = paths[Path] extends {
  [K in Method]: { requestBody: { content: { 'application/json': infer B } } };
}
  ? B
  : never;

/** The JSON response of the first 2xx status the operation documents. */
type JsonResponse<
  Path extends keyof paths,
  Method extends 'get' | 'post' | 'patch' | 'delete',
> = paths[Path] extends {
  [K in Method]: { responses: { 200: { content: { 'application/json': infer R } } } };
}
  ? R
  : paths[Path] extends {
        [K in Method]: { responses: { 201: { content: { 'application/json': infer R } } } };
      }
    ? R
    : paths[Path] extends {
          [K in Method]: { responses: { 202: { content: { 'application/json': infer R } } } };
        }
      ? R
      : unknown;

/** Query parameters of the `GET` operation at `Path`. */
type QueryParams<Path extends keyof paths> = paths[Path] extends {
  get: { parameters: { query?: infer Q } };
}
  ? Q
  : never;

/** The documented path whose `{placeholder}` segments are spelled `Params`. */
type TemplatePath<Params extends Record<string, string>> = {
  [Path in keyof paths & string]: Path extends string
    ? string extends Path
      ? never
      : Params extends Record<string, string>
        ? Path extends `/${string}`
          ? Path
          : never
        : never
    : never;
}[keyof paths & string];

/** Render query parameters into a query string, dropping empty values. */
const buildQueryString = (params?: Record<string, unknown>): string => {
  if (!params) return '';
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null || value === '') continue;
    search.set(key, Array.isArray(value) ? value.join(',') : String(value));
  }
  const query = search.toString();
  return query ? `?${query}` : '';
};

/**
 * Fill the `{placeholder}` segments of a documented path.
 *
 * FastAPI generates paths like `/api/v1/jobs/{job_id}`, which are not valid
 * `keyof paths` template-literal types. This replaces each placeholder and
 * throws when one is missing, so a typo fails loudly at the call site.
 */
export const fillPath = <Params extends Record<string, string>>(
  template: string,
  params: Params,
): string =>
  template.replace(/\{(\w+)\}/g, (_, name: string) => {
    const value = params[name];
    if (value === undefined) {
      throw new Error(`Missing path parameter "${name}" for ${template}`);
    }
    return encodeURIComponent(value);
  });

/** `GET` a documented path with its schema-typed response. */
export const apiGetTyped = <Path extends keyof paths>(
  path: Path,
  query?: QueryParams<Path>,
  init?: ApiRequestOptions,
): Promise<JsonResponse<Path, 'get'>> =>
  apiGet<JsonResponse<Path, 'get'>>(
    `${String(path)}${buildQueryString(query as Record<string, unknown>)}`,
    init,
  );

/** `GET` a documented path that carries `{placeholder}` parameters. */
export const apiGetPath = <Path extends keyof paths & string>(
  template: Path,
  params: Record<string, string>,
  init?: ApiRequestOptions,
): Promise<JsonResponse<Path, 'get'>> =>
  apiGet<JsonResponse<Path, 'get'>>(fillPath(template, params), init);

/** `POST` a documented path with a schema-typed body and response. */
export const apiPostTyped = <Path extends keyof paths>(
  path: Path,
  body: JsonBody<Path, 'post'>,
  init?: ApiRequestOptions,
): Promise<JsonResponse<Path, 'post'>> =>
  apiPost<JsonResponse<Path, 'post'>>(String(path), body, init);

/** `PATCH` a documented path with a schema-typed body and response. */
export const apiPatchTyped = <Path extends keyof paths>(
  path: Path,
  body: JsonBody<Path, 'patch'>,
  init?: ApiRequestOptions,
): Promise<JsonResponse<Path, 'patch'>> =>
  apiPatch<JsonResponse<Path, 'patch'>>(String(path), body, init);

/** `DELETE` a documented path; the contract returns no body. */
export const apiDeleteTyped = <Path extends keyof paths>(
  path: Path,
  init?: ApiRequestOptions,
): Promise<void> => apiRequest<void>(String(path), { method: 'DELETE', ...init }) as Promise<void>;

export type { paths, TemplatePath };
