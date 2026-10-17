import fs from 'node:fs';
import path from 'node:path';

export type Doc = {
  slug: string;
  title: string;
  description?: string;
  author?: string;
  order?: number;
  content: string;
};

const docsDirectory = path.join(process.cwd(), 'docs');
const docFilePattern = /^[a-z0-9-]+\.md$/i;

function parseFrontmatter(raw: string): {
  data: Record<string, string>;
  body: string;
} {
  if (!raw.startsWith('---')) {
    return { data: {}, body: raw };
  }
  const parts = raw.split('---');
  if (parts.length < 3) {
    return { data: {}, body: raw };
  }
  const frontmatter = parts[1];
  const body = parts.slice(2).join('---').replace(/^\n+/, '');
  const data: Record<string, string> = {};
  for (const line of frontmatter.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const idx = trimmed.indexOf(':');
    if (idx === -1) continue;
    const key = trimmed.slice(0, idx).trim();
    let value = trimmed.slice(idx + 1).trim();
    value = value.replace(/^['"]|['"]$/g, '');
    data[key] = value;
  }
  return { data, body };
}

function readDoc(fileName: string): Doc {
  const slug = fileName.replace(/\.md$/i, '');
  const raw = fs.readFileSync(path.join(docsDirectory, fileName), 'utf8');
  const { data, body } = parseFrontmatter(raw);
  const heading = body.match(/^#\s+(.+)$/m)?.[1]?.trim();
  const title =
    data.title?.trim() ||
    heading ||
    slug
      .split('-')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');

  const order = data.order !== undefined ? Number(data.order) : undefined;

  return {
    slug,
    title,
    description: data.description,
    author: data.author,
    order: Number.isNaN(order as number) ? undefined : order,
    content: body,
  };
}

export function getDocs(): Doc[] {
  return fs
    .readdirSync(docsDirectory)
    .filter((fileName) => docFilePattern.test(fileName) && fileName.toLowerCase() !== 'index.md')
    .map(readDoc)
    .sort((left, right) => {
      if (left.order !== undefined && right.order !== undefined) {
        return left.order - right.order;
      }
      if (left.order !== undefined) return -1;
      if (right.order !== undefined) return 1;
      return left.title.localeCompare(right.title, undefined, { numeric: true });
    });
}

export function getDoc(slug: string): Doc | undefined {
  if (!/^[a-z0-9-]+$/i.test(slug)) return undefined;
  const fileName = `${slug}.md`;
  if (!fs.existsSync(path.join(docsDirectory, fileName))) return undefined;
  return readDoc(fileName);
}
