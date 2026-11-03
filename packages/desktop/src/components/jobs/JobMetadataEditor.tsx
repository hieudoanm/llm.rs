'use client';

import { useState } from 'react';
import { ApiAlert } from '@/components/ui/ApiAlert';
import { ApiError } from '@/lib/api';
import { MAX_NAME_LENGTH, MAX_TAGS, MAX_TAG_LENGTH } from '@/lib/constants';
import { PiPlus, PiX } from 'react-icons/pi';

export interface JobMetadataEditorProps {
  jobId: string;
  name: string | null;
  tags: string[];
  /** Called after the metadata was saved. */
  onSaved?: (metadata: { name: string | null; tags: string[] }) => void;
  /** Patches the job; injected so the component stays testable. */
  onSave: (metadata: {
    name: string | null;
    tags: string[];
  }) => Promise<{ name: string | null; tags: string[] }>;
}

/** The tag rules of the backend, applied on the client as well. */
export const normaliseTag = (raw: string): string => raw.trim().toLowerCase();

/** True when the tag may be stored. */
export const isValidTag = (tag: string): boolean => {
  if (!tag || tag.length > MAX_TAG_LENGTH) return false;
  return /^[a-z0-9._\- ]+$/.test(tag);
};

/**
 * The editor of the human-readable job metadata.
 *
 * Only the name and the tags are editable; the training configuration of a job
 * is immutable once submitted.
 */
export const JobMetadataEditor = ({
  jobId,
  name,
  tags,
  onSaved,
  onSave,
}: JobMetadataEditorProps) => {
  const [draftName, setDraftName] = useState(name ?? '');
  const [draftTags, setDraftTags] = useState<string[]>(tags);
  const [tagDraft, setTagDraft] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const addTag = () => {
    const tag = normaliseTag(tagDraft);
    if (!tag) return;
    if (!isValidTag(tag)) {
      setError(
        `A tag may hold letters, digits, spaces, and - _ . (max ${MAX_TAG_LENGTH} characters).`,
      );
      return;
    }
    if (draftTags.includes(tag)) {
      setTagDraft('');
      return;
    }
    if (draftTags.length >= MAX_TAGS) {
      setError(`A job may carry at most ${MAX_TAGS} tags.`);
      return;
    }
    setDraftTags([...draftTags, tag]);
    setTagDraft('');
    setError(null);
  };

  const removeTag = (tag: string) => {
    setDraftTags(draftTags.filter((entry) => entry !== tag));
  };

  const save = async () => {
    setSaving(true);
    setError(null);
    try {
      const result = await onSave({
        name: draftName.trim() ? draftName.trim() : null,
        tags: draftTags,
      });
      setSaved(true);
      onSaved?.(result);
    } catch (cause) {
      setSaved(false);
      setError(cause instanceof ApiError ? cause.message : 'The metadata could not be saved.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className='mb-6' aria-label='Job name and tags'>
      <h2 className='text-text-primary mb-3 text-lg font-semibold'>Name &amp; tags</h2>

      <div className='flex flex-col gap-3'>
        <label className='form-control max-w-sm'>
          <span className='label-text mb-1 text-sm'>Name</span>
          <input
            type='text'
            className='input input-sm w-full'
            value={draftName}
            maxLength={MAX_NAME_LENGTH}
            placeholder='e.g. baseline run'
            onChange={(event) => {
              setDraftName(event.target.value);
              setSaved(false);
            }}
          />
        </label>

        <div>
          <span className='label-text mb-1 block text-sm'>Tags</span>
          <ul className='mb-2 flex flex-wrap items-center gap-2'>
            {draftTags.map((tag) => (
              <li key={tag}>
                <span className='badge badge-outline gap-1'>
                  {tag}
                  <button
                    type='button'
                    onClick={() => {
                      removeTag(tag);
                      setSaved(false);
                    }}
                    aria-label={`Remove tag ${tag}`}>
                    <PiX className='icon-xs' aria-hidden='true' />
                  </button>
                </span>
              </li>
            ))}
            {draftTags.length === 0 && <li className='text-text-tertiary text-xs'>No tags yet.</li>}
          </ul>

          <div className='flex items-center gap-2'>
            <input
              type='text'
              className='input input-sm w-48'
              value={tagDraft}
              placeholder='Add a tag'
              aria-label='Add a tag'
              onChange={(event) => setTagDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  addTag();
                }
              }}
            />
            <button type='button' className='btn btn-sm' onClick={addTag}>
              <PiPlus className='icon-sm' aria-hidden='true' />
              Add
            </button>
          </div>
        </div>

        {error && <ApiAlert variant='error' message={error} />}

        <div className='flex items-center gap-3'>
          <button
            type='button'
            className='btn btn-primary btn-sm'
            onClick={save}
            disabled={saving}
            data-job-id={jobId}>
            {saving ? 'Saving…' : 'Save'}
          </button>
          {saved && !error && (
            <span className='text-success text-xs' role='status'>
              Saved.
            </span>
          )}
        </div>
      </div>
    </section>
  );
};
