import type { Metadata, NextPage } from 'next';
import type { FC } from 'react';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuActivity,
  LuBot,
  LuDownload,
  LuGrid3X3,
  LuLayers,
  LuLock,
  LuMonitor,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/llm.rs';

const FEATURES: Feature[] = [
  {
    icon: LuGrid3X3,
    title: 'Six task types',
    body: 'Classification, regression, semi-supervised, clustering, anomaly detection, and offline RL share one workflow.',
  },
  {
    icon: LuLock,
    title: 'Local-first and private',
    body: 'No uploads, no API keys, no accounts. Everything persists to a local SQLite database on your disk.',
  },
  {
    icon: LuDownload,
    title: 'ONNX is the contract',
    body: 'Every successful job exports a portable .onnx model that runs in any ONNX-compatible runtime, independent of the training code.',
  },
  {
    icon: LuMonitor,
    title: 'Two surfaces, one engine',
    body: 'A native desktop app for the visual workflow, or the llm CLI for scripts and headless training — same Rust core underneath.',
  },
  {
    icon: LuActivity,
    title: 'Jobs and metrics',
    body: 'Every training run is a tracked job with accuracy, F1, and loss metrics you can inspect and compare.',
  },
  {
    icon: LuLayers,
    title: 'Offline reinforcement learning',
    body: 'Train RL policies without a simulator service — environments run locally alongside the model.',
  },
  {
    icon: LuBot,
    title: 'MCP server',
    body: 'llm mcp exposes datasets, algorithms, training jobs, and ONNX export to MCP clients — agents can run the full local ML workflow over stdio.',
  },
];

const HERO_LINES = [
  '$ llm dataset data/iris.csv',
  'registered dataset 3f2a91c7 (150 rows, 5 columns)',
  '$ llm train --task classification …',
  'job 8c1d04e2 completed · accuracy 0.973',
  '$ llm export --job-id 8c1d04e2 --output model.onnx',
  'wrote model.onnx (12.4 KB)',
];

const DEMO_TERMINALS: {
  caption: string;
  lines: string[];
  wide?: boolean;
}[] = [
  {
    caption: 'Train a classifier',
    wide: true,
    lines: [
      '$ llm train --task classification \\',
      '    --algorithm logistic_regression \\',
      '    --features sepal_length,sepal_width,petal_length,petal_width \\',
      '    --label species',
      'job 8c1d04e2 completed · accuracy 0.973 · f1 0.968',
    ],
  },
  {
    caption: 'Register a dataset',
    lines: [
      '$ llm dataset data/iris.csv',
      'registered dataset 3f2a91c7 (150 rows, 5 columns)',
    ],
  },
  {
    caption: 'Export ONNX',
    lines: [
      '$ llm export --job-id 8c1d04e2 --output model.onnx',
      'wrote model.onnx (12.4 KB)',
    ],
  },
];

const STEPS = [
  {
    title: 'Register a dataset',
    body: 'Point llm at a CSV file. Columns, types, and labels are inferred and stored in the local dataset registry.',
    snippet: 'llm dataset data/iris.csv',
  },
  {
    title: 'Train a job',
    body: 'Pick a task and algorithm, choose features and label — the job runs locally and records metrics as it goes.',
    snippet: 'llm train --task classification …',
  },
  {
    title: 'Export ONNX',
    body: 'A successful job exports a portable .onnx model that runs in any ONNX-compatible runtime — no Rust required downstream.',
    snippet: 'llm export --job-id … --output model.onnx',
  },
];

const INSTALL_LINES = [
  '$ cd crates',
  '$ cargo build --release -p llm-cli',
  '$ ./target/release/llm dataset data/iris.csv',
  '$ ./target/release/llm train --task classification …',
  '$ ./target/release/llm export --job-id <id> --output model.onnx',
];

const StepCard: FC<{ step: (typeof STEPS)[number]; index: number }> = ({
  step,
  index,
}) => (
  <article className="card border-base-300 bg-base-100 rounded-2xl border p-6">
    <span className="text-accent mb-2 block font-mono text-sm font-semibold">
      {String(index + 1).padStart(2, '0')}
    </span>
    <h3 className="text-base-content m-0 mb-2 text-lg font-bold">
      {step.title}
    </h3>
    <p className="text-base-content/60 text-sm">{step.body}</p>
    <code className="border-base-300 bg-base-200 text-base-content/60 mt-4 block overflow-x-auto rounded-lg border px-3 py-2 font-mono text-xs">
      {step.snippet}
    </code>
  </article>
);

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="Train ML models from CSV, on your own machine."
      description="llm.rs handles classification, regression, semi-supervised self-training, clustering, anomaly detection, and offline reinforcement learning — then exports a portable ONNX model. No cloud, no account, no notebooks."
      primary={{ label: 'Get started', href: '#install' }}
      secondary={{ label: 'See it in action', href: '#demo' }}
      stats={[
        { value: '6', label: 'task types' },
        { value: '0', label: 'cloud calls' },
        { value: 'ONNX', label: 'portable export' },
      ]}>
      <Terminal lines={HERO_LINES} />
    </Hero>

    <Features
      title="Everything happens locally."
      description="Your CSV and trained artefacts never leave your machine. Datasets, jobs, metrics, and models persist to local SQLite storage."
      features={FEATURES}
    />

    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="See it in action"
          description="Register a CSV, train a model, export ONNX — three commands, all local."
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal
              key={t.caption}
              lines={t.lines}
              className={t.wide ? 'lg:col-span-2' : ''}
            />
          ))}
        </div>
      </div>
    </section>

    <section id="how-it-works" className="py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="How the CLI works"
          description="CSV in, ONNX out — a three-step local pipeline."
        />
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {STEPS.map((step, i) => (
            <StepCard key={step.title} step={step} index={i} />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-3xl px-5">
        <SectionHead
          title="Get started"
          description="Desktop builds for macOS, Windows, and Linux are on the releases page. Or build the CLI from source with Cargo."
        />
        <Terminal lines={INSTALL_LINES} />
        <p className="text-base-content/60 mt-6 text-center text-sm">
          Prebuilt desktop bundles are on the{' '}
          <a
            href={`${GITHUB}/releases`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            releases page
          </a>{' '}
          — see the{' '}
          <a
            href={`${GITHUB}#readme`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            README
          </a>{' '}
          for details.
        </p>
      </div>
    </section>

    <CTA
      title="Ready to train locally?"
      description="CSV in, ONNX out — no cloud required."
      primary={{ label: 'View on GitHub', href: GITHUB, external: true }}
      secondary={{
        label: 'Get the desktop app',
        href: `${GITHUB}/releases`,
        external: true,
      }}
    />
  </main>
);

export default HomePage;
