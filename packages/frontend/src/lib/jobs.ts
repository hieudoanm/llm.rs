/**
 * Job discovery helpers (FEATURE-13).
 *
 * The filter set is a plain object so it can be compared, serialised into the
 * URL query string, and turned into the removable chips shown above the table.
 */

export const JOB_STATUSES = [
  'queued',
  'validating',
  'training',
  'evaluating',
  'exporting',
  'retrying',
  'completed',
  'failed',
] as const;

export const JOB_TASK_TYPES = [
  'classification',
  'regression',
  'semi_supervised',
  'clustering',
  'anomaly_detection',
  'reinforcement_learning',
] as const;

export const SORT_FIELDS = ['createdAt', 'updatedAt', 'status', 'algorithmId'] as const;

export type SortField = (typeof SORT_FIELDS)[number];
export type SortDirection = 'asc' | 'desc';

export interface JobFilters {
  status: string[];
  taskType: string[];
  algorithmId: string[];
  tag: string[];
  q: string;
  createdFrom: string;
  createdTo: string;
  sort: string;
}

export const EMPTY_FILTERS: JobFilters = {
  status: [],
  taskType: [],
  algorithmId: [],
  tag: [],
  q: '',
  createdFrom: '',
  createdTo: '',
  sort: '',
};

/** Split a comma-separated parameter into its trimmed, non-empty values. */
export const splitParam = (value: string | null | undefined): string[] =>
  (value ?? '')
    .split(',')
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);

/**
 * Build the query string of a filter set.
 *
 * Empty values are omitted, so the query string only carries what is set.
 */
export const toQueryString = (filters: JobFilters): string => {
  const params = new URLSearchParams();
  if (filters.status.length) params.set('status', filters.status.join(','));
  if (filters.taskType.length) params.set('taskType', filters.taskType.join(','));
  if (filters.algorithmId.length) params.set('algorithmId', filters.algorithmId.join(','));
  if (filters.tag.length) params.set('tag', filters.tag.join(','));
  if (filters.q.trim()) params.set('q', filters.q.trim());
  if (filters.createdFrom) params.set('createdFrom', filters.createdFrom);
  if (filters.createdTo) params.set('createdTo', filters.createdTo);
  if (filters.sort) params.set('sort', filters.sort);
  return params.toString();
};

/** Read a filter set out of a query string. */
export const fromQueryString = (search: string): JobFilters => {
  const params = new URLSearchParams(search);
  return {
    status: splitParam(params.get('status')),
    taskType: splitParam(params.get('taskType')),
    algorithmId: splitParam(params.get('algorithmId')),
    tag: splitParam(params.get('tag')),
    q: params.get('q') ?? '',
    createdFrom: params.get('createdFrom') ?? '',
    createdTo: params.get('createdTo') ?? '',
    sort: params.get('sort') ?? '',
  };
};

/**
 * Turn a filter set into the structured query parameters of the typed client.
 *
 * Multi-value filters become comma-separated strings (the API contract), and
 * an unset filter is omitted so the request carries only what is set.
 */
export const toQueryParams = (
  filters: JobFilters,
): {
  status?: string;
  taskType?: string;
  algorithmId?: string;
  tag?: string;
  q?: string;
  createdFrom?: string;
  createdTo?: string;
  sort?: string;
} => {
  const params: Record<string, string> = {};
  if (filters.status.length) params.status = filters.status.join(',');
  if (filters.taskType.length) params.taskType = filters.taskType.join(',');
  if (filters.algorithmId.length) params.algorithmId = filters.algorithmId.join(',');
  if (filters.tag.length) params.tag = filters.tag.join(',');
  if (filters.q.trim()) params.q = filters.q.trim();
  if (filters.createdFrom) params.createdFrom = filters.createdFrom;
  if (filters.createdTo) params.createdTo = filters.createdTo;
  if (filters.sort) params.sort = filters.sort;
  return params;
};

/** The `localStorage` key under which the last-used filter set is kept. */
export const FILTERS_STORAGE_KEY = 'mlrs-jobs-filters';

/**
 * Read the last-used filter set from `localStorage`.
 *
 * Returns `null` when nothing is stored, the stored value is unreadable, or
 * storage is unavailable (`localStorage` throws in private mode and in SSR).
 * A filter set is a flat object of strings and string arrays, so a malformed
 * value is discarded rather than trusted.
 */
export const readStoredFilters = (): JobFilters | null => {
  if (typeof window === 'undefined') return null;
  try {
    const raw = window.localStorage.getItem(FILTERS_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<JobFilters>;
    if (typeof parsed !== 'object' || parsed === null) return null;
    return {
      status: Array.isArray(parsed.status) ? parsed.status : [],
      taskType: Array.isArray(parsed.taskType) ? parsed.taskType : [],
      algorithmId: Array.isArray(parsed.algorithmId) ? parsed.algorithmId : [],
      tag: Array.isArray(parsed.tag) ? parsed.tag : [],
      q: typeof parsed.q === 'string' ? parsed.q : '',
      createdFrom: typeof parsed.createdFrom === 'string' ? parsed.createdFrom : '',
      createdTo: typeof parsed.createdTo === 'string' ? parsed.createdTo : '',
      sort: typeof parsed.sort === 'string' ? parsed.sort : '',
    };
  } catch {
    return null;
  }
};

/** Persist the filter set; a failed write (private mode) is not fatal. */
export const writeStoredFilters = (filters: JobFilters): void => {
  if (typeof window === 'undefined') return;
  try {
    if (!hasActiveFilters(filters)) {
      window.localStorage.removeItem(FILTERS_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(FILTERS_STORAGE_KEY, JSON.stringify(filters));
  } catch {
    // Ignore: the filters still live in the URL for this session.
  }
};

export interface FilterChip {
  /** The filter group the chip belongs to. */
  group: keyof Omit<JobFilters, 'q' | 'createdFrom' | 'createdTo' | 'sort'>;
  /** The value the chip stands for. */
  value: string;
  /** The label shown on the chip. */
  label: string;
}

const GROUP_LABELS: Record<string, string> = {
  status: 'Status',
  taskType: 'Task',
  algorithmId: 'Algorithm',
  tag: 'Tag',
};

/** The removable chips that describe the active filter set. */
export const filterChips = (filters: JobFilters): FilterChip[] => {
  const chips: FilterChip[] = [];
  for (const group of ['status', 'taskType', 'algorithmId', 'tag'] as const) {
    for (const value of filters[group]) {
      chips.push({ group, value, label: `${GROUP_LABELS[group]}: ${value}` });
    }
  }
  if (filters.q.trim()) chips.push({ group: 'status', value: '', label: `Search: ${filters.q}` });
  if (filters.createdFrom) {
    chips.push({ group: 'status', value: '', label: `From: ${filters.createdFrom}` });
  }
  if (filters.createdTo)
    chips.push({ group: 'status', value: '', label: `To: ${filters.createdTo}` });
  if (filters.sort) chips.push({ group: 'status', value: '', label: `Sort: ${filters.sort}` });
  return chips;
};

/** True when no filter is set. */
export const hasActiveFilters = (filters: JobFilters): boolean => filterChips(filters).length > 0;

/** Remove one chip from the filter set. */
export const removeChip = (filters: JobFilters, chip: FilterChip): JobFilters => {
  if (chip.group === 'status' && chip.value === '') {
    // The synthetic chips (search, date range, sort) are removed by key.
    if (chip.label.startsWith('Search:')) return { ...filters, q: '' };
    if (chip.label.startsWith('From:')) return { ...filters, createdFrom: '' };
    if (chip.label.startsWith('To:')) return { ...filters, createdTo: '' };
    if (chip.label.startsWith('Sort:')) return { ...filters, sort: '' };
    return filters;
  }
  return {
    ...filters,
    [chip.group]: filters[chip.group].filter((entry) => entry !== chip.value),
  };
};

/** Toggle one value of a multi-select filter group. */
export const toggleFilterValue = (
  filters: JobFilters,
  group: 'status' | 'taskType' | 'algorithmId' | 'tag',
  value: string,
): JobFilters => {
  const current = filters[group];
  return {
    ...filters,
    [group]: current.includes(value)
      ? current.filter((entry) => entry !== value)
      : [...current, value],
  };
};

/** Parse a `sort` value into its field and direction. */
export const parseSort = (sort: string): { field: SortField; direction: SortDirection } => {
  const [rawField, rawDirection] = sort.split(':');
  const field = (SORT_FIELDS as readonly string[]).includes(rawField)
    ? (rawField as SortField)
    : 'createdAt';
  const direction: SortDirection = rawDirection === 'asc' ? 'asc' : 'desc';
  return { field, direction };
};

/** The `sort` value that toggles a column header. */
export const nextSort = (current: string, field: SortField): string => {
  // A time column starts with the newest first; the other columns start with
  // the natural ascending order.
  const natural = field === 'createdAt' || field === 'updatedAt' ? `${field}:desc` : `${field}:asc`;
  if (!current) return natural;

  const { field: currentField, direction } = parseSort(current);
  if (currentField !== field) return natural;
  return `${field}:${direction === 'asc' ? 'desc' : 'asc'}`;
};
