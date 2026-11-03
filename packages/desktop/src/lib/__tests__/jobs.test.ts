import { beforeEach, describe, expect, it } from 'vitest';
import {
  EMPTY_FILTERS,
  FILTERS_STORAGE_KEY,
  filterChips,
  fromQueryString,
  hasActiveFilters,
  nextSort,
  parseSort,
  readStoredFilters,
  removeChip,
  splitParam,
  toQueryParams,
  toQueryString,
  toggleFilterValue,
  writeStoredFilters,
  type JobFilters,
} from '../jobs';

const filters: JobFilters = {
  status: ['completed', 'failed'],
  taskType: ['classification'],
  algorithmId: [],
  tag: ['baseline'],
  q: 'iris',
  createdFrom: '',
  createdTo: '',
  sort: 'createdAt:desc',
};

describe('splitParam', () => {
  it('splits a comma separated list and drops the blanks', () => {
    expect(splitParam('a, b ,,c')).toEqual(['a', 'b', 'c']);
  });

  it('returns an empty list for a missing value', () => {
    expect(splitParam(null)).toEqual([]);
    expect(splitParam(undefined)).toEqual([]);
    expect(splitParam('')).toEqual([]);
  });
});

describe('toQueryString', () => {
  it('serialises every active filter', () => {
    const query = toQueryString(filters);

    expect(query).toContain('status=completed%2Cfailed');
    expect(query).toContain('taskType=classification');
    expect(query).toContain('tag=baseline');
    expect(query).toContain('q=iris');
    expect(query).toContain('sort=createdAt%3Adesc');
  });

  it('omits the empty filters', () => {
    expect(toQueryString(EMPTY_FILTERS)).toBe('');
  });
});

describe('fromQueryString', () => {
  it('reads every filter back', () => {
    expect(fromQueryString(toQueryString(filters))).toEqual(filters);
  });

  it('returns the empty filters for an empty query string', () => {
    expect(fromQueryString('')).toEqual(EMPTY_FILTERS);
  });
});

describe('filterChips', () => {
  it('describes every active filter', () => {
    const chips = filterChips(filters);
    const labels = chips.map((chip) => chip.label);

    expect(labels).toContain('Status: completed');
    expect(labels).toContain('Status: failed');
    expect(labels).toContain('Task: classification');
    expect(labels).toContain('Tag: baseline');
    expect(labels).toContain('Search: iris');
    expect(labels).toContain('Sort: createdAt:desc');
  });

  it('returns nothing when no filter is active', () => {
    expect(filterChips(EMPTY_FILTERS)).toEqual([]);
    expect(hasActiveFilters(EMPTY_FILTERS)).toBe(false);
  });
});

describe('removeChip', () => {
  it('removes one value of a multi-select group', () => {
    const chip = filterChips(filters).find((entry) => entry.label === 'Status: completed');
    const next = removeChip(filters, chip!);

    expect(next.status).toEqual(['failed']);
    expect(next.tag).toEqual(['baseline']);
  });

  it('removes the search term', () => {
    const chip = filterChips(filters).find((entry) => entry.label === 'Search: iris');

    expect(removeChip(filters, chip!).q).toBe('');
  });

  it('removes the sort order', () => {
    const chip = filterChips(filters).find((entry) => entry.label.startsWith('Sort:'));

    expect(removeChip(filters, chip!).sort).toBe('');
  });

  it('removes the date range', () => {
    const dated = { ...filters, createdFrom: '2026-01-01', createdTo: '2026-02-01' };
    const from = filterChips(dated).find((entry) => entry.label.startsWith('From:'));
    const to = filterChips(dated).find((entry) => entry.label.startsWith('To:'));

    expect(removeChip(dated, from!).createdFrom).toBe('');
    expect(removeChip(dated, to!).createdTo).toBe('');
  });
});

describe('toggleFilterValue', () => {
  it('adds a value that is not selected', () => {
    const next = toggleFilterValue(EMPTY_FILTERS, 'status', 'queued');

    expect(next.status).toEqual(['queued']);
  });

  it('removes a value that is already selected', () => {
    const next = toggleFilterValue(filters, 'status', 'completed');

    expect(next.status).toEqual(['failed']);
  });
});

describe('parseSort', () => {
  it('reads the field and the direction', () => {
    expect(parseSort('createdAt:asc')).toEqual({ field: 'createdAt', direction: 'asc' });
  });

  it('defaults to the newest first', () => {
    expect(parseSort('')).toEqual({ field: 'createdAt', direction: 'desc' });
    expect(parseSort('nonsense:sideways')).toEqual({ field: 'createdAt', direction: 'desc' });
  });
});

describe('nextSort', () => {
  it('starts a new field with the natural direction', () => {
    expect(nextSort('createdAt:desc', 'algorithmId')).toBe('algorithmId:asc');
    expect(nextSort('createdAt:desc', 'updatedAt')).toBe('updatedAt:desc');
  });

  it('starts with the newest first when nothing is sorted yet', () => {
    expect(nextSort('', 'createdAt')).toBe('createdAt:desc');
    expect(nextSort('', 'algorithmId')).toBe('algorithmId:asc');
  });

  it('flips the direction of the active field', () => {
    expect(nextSort('createdAt:desc', 'createdAt')).toBe('createdAt:asc');
    expect(nextSort('createdAt:asc', 'createdAt')).toBe('createdAt:desc');
  });
});

describe('toQueryParams', () => {
  it('omits an unset filter', () => {
    expect(toQueryParams(EMPTY_FILTERS)).toEqual({});
  });

  it('joins multi-value filters with commas', () => {
    const filters: JobFilters = {
      ...EMPTY_FILTERS,
      status: ['queued', 'failed'],
      tag: ['nightly'],
      q: 'baseline',
      sort: 'createdAt:asc',
    };

    expect(toQueryParams(filters)).toEqual({
      status: 'queued,failed',
      tag: 'nightly',
      q: 'baseline',
      sort: 'createdAt:asc',
    });
  });

  it('trims the search term and drops a blank one', () => {
    expect(toQueryParams({ ...EMPTY_FILTERS, q: '   ' })).toEqual({});
    expect(toQueryParams({ ...EMPTY_FILTERS, q: '  km  ' })).toEqual({ q: 'km' });
  });
});

describe('stored filters', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it('returns null when nothing is stored', () => {
    expect(readStoredFilters()).toBeNull();
  });

  it('round-trips an active filter set', () => {
    const filters: JobFilters = { ...EMPTY_FILTERS, status: ['failed'], q: 'km' };

    writeStoredFilters(filters);

    expect(readStoredFilters()).toEqual(filters);
  });

  it('clears the entry when no filter is active', () => {
    writeStoredFilters({ ...EMPTY_FILTERS, status: ['failed'] });
    writeStoredFilters(EMPTY_FILTERS);

    expect(window.localStorage.getItem(FILTERS_STORAGE_KEY)).toBeNull();
  });

  it('discards an unreadable value', () => {
    window.localStorage.setItem(FILTERS_STORAGE_KEY, 'not json');

    expect(readStoredFilters()).toBeNull();
  });

  it('coerces a partially valid value into a complete filter set', () => {
    window.localStorage.setItem(
      FILTERS_STORAGE_KEY,
      JSON.stringify({ status: ['queued'], q: 7, taskType: 'classification' }),
    );

    expect(readStoredFilters()).toEqual({
      ...EMPTY_FILTERS,
      status: ['queued'],
    });
  });

  it('discards a JSON null value', () => {
    window.localStorage.setItem(FILTERS_STORAGE_KEY, 'null');

    expect(readStoredFilters()).toBeNull();
  });

  it('is a no-op when storage throws', () => {
    const setItem = window.localStorage.setItem.bind(window.localStorage);
    window.localStorage.setItem = () => {
      throw new Error('QuotaExceededError');
    };

    expect(() => writeStoredFilters({ ...EMPTY_FILTERS, q: 'km' })).not.toThrow();

    window.localStorage.setItem = setItem;
  });

  it('returns null when reading throws', () => {
    window.localStorage.getItem = () => {
      throw new Error('SecurityError');
    };

    expect(readStoredFilters()).toBeNull();
  });
});
