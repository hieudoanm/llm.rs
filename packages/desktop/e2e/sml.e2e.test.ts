import { test, expect } from '@playwright/test';
import type { Page } from '@playwright/test';

test.setTimeout(120000);

const uploadFile = async (page: Page, csvContent: string, fileName: string) => {
  const fileInput = page.locator('input[type="file"]');
  await fileInput.waitFor({ state: 'attached' });
  await page.evaluate(
    async ([csv, name]) => {
      const input = document.querySelector('input[type="file"]') as HTMLInputElement;
      input.style.opacity = '1';
      input.style.position = 'static';
      input.style.zIndex = '9999';
      input.style.width = 'auto';
      input.style.height = 'auto';
      input.style.margin = '0';
      input.style.padding = '0';
      input.style.border = 'none';
      input.style.background = 'none';
      input.style.display = 'block';
      input.removeAttribute('disabled');
      const blob = new Blob([csv], { type: 'text/csv' });
      const file = new File([blob], name, { type: 'text/csv' });
      const dataTransfer = new DataTransfer();
      dataTransfer.items.add(file);
      input.files = dataTransfer.files;
      input.dispatchEvent(new Event('change', { bubbles: true }));
    },
    [csvContent, fileName],
  );
};

const CLASSIFICATION_CSV = `feature1,feature2,feature3,label
1.2,3.4,red,0
2.1,5.6,blue,1
3.5,2.1,red,0
4.2,1.8,blue,1
5.1,4.3,red,0
6.3,3.9,blue,1
7.4,2.7,red,0
8.1,4.1,blue,1`;

const REGRESSION_CSV = `sqft,bedrooms,age,price
1500,3,10,350000
2000,4,5,450000
1200,2,15,280000
1800,3,8,380000
2200,4,3,480000
1600,3,12,320000
1900,3,7,410000
2100,4,6,440000`;

const SEMI_SUPERVISED_CSV = `feature1,feature2,feature3,label
1.2,3.4,red,0
2.1,5.6,blue,1
3.5,2.1,red,0
4.2,1.8,blue,1
5.1,4.3,red,0
6.3,3.9,blue,1
7.4,2.7,red,0
8.1,4.1,blue,1
1.6,3.1,red,0
2.7,5.1,blue,1
3.9,2.6,red,0
4.8,1.2,blue,1
5.7,4.8,green,
6.9,3.2,green,
7.8,2.2,green,
8.6,4.6,green,`;

const createJobViaAPI = async (
  page: Page,
  datasetId: string,
  taskType: string,
  algorithmId: string,
  hyperparameters: Record<string, unknown> = {},
) => {
  const result = await page.evaluate(
    async ({ datasetId, taskType, algorithmId, hyperparameters }) => {
      const unsupervised = taskType === 'clustering' || taskType === 'anomaly_detection';
      const regression = taskType === 'regression';
      const response = await fetch('http://127.0.0.1:8000/api/v1/jobs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          datasetId,
          featureColumns: regression
            ? ['sqft', 'bedrooms', 'age']
            : ['feature1', 'feature2', 'feature3'],
          ...(unsupervised ? {} : { labelColumn: regression ? 'price' : 'label' }),
          taskType,
          algorithmId,
          hyperparameters,
          exportPkl: true,
          exportJoblib: true,
        }),
      });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(`Job creation failed: ${response.status} ${text}`);
      }
      return await response.json();
    },
    { datasetId, taskType, algorithmId, hyperparameters },
  );
  return result.jobId;
};

const waitForJobCompletion = async (page: Page, jobId: string) => {
  for (let i = 0; i < 60; i++) {
    const res = await page.request.fetch(`http://127.0.0.1:8000/api/v1/jobs/${jobId}`);
    const job = await res.json();
    if (job.status === 'completed' || job.status === 'failed') return job;
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error(`Job ${jobId} did not complete in time`);
};

const verifyArtefacts = async (page: Page, jobId: string) => {
  const res = await page.request.fetch(`http://127.0.0.1:8000/api/v1/jobs/${jobId}`);
  const job = await res.json();
  const formats = job.artefacts.map((a: { format: string }) => a.format);
  expect(formats).toContain('onnx');
  expect(formats).toContain('pkl');
  expect(formats).toContain('joblib');
  return job;
};

test.describe('Demo Scene 1: Title & Branding', () => {
  test('home page shows llm branding and feature grid', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toContainText('llm-client');
    await expect(page.getByRole('link', { name: 'Train a model' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'View jobs' })).toBeVisible();
    await expect(page.getByText('CSV upload')).toBeVisible();
    await expect(page.getByText('Task-aware models')).toBeVisible();
    await expect(page.getByText('ONNX export')).toBeVisible();
    await expect(page.getByText('Upload dataset')).toBeVisible();
    await expect(page.getByText('Map and configure')).toBeVisible();
    await expect(page.getByText('Train and download')).toBeVisible();
    await expect(page.getByText('Ready to train your first model?')).toBeVisible();
  });
});

test.describe('Demo Scene 2: Theme Toggle', () => {
  test('toggles between llm-light and llm-dark themes', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'llm-light');
    const toggle = page.getByLabel('Toggle theme');
    await toggle.click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'llm-dark');
    await toggle.click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'llm-light');
  });
});

test.describe('Demo Scene 3: Navbar Navigation', () => {
  test('navigates between logo, new job, jobs, and algorithms', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: 'Train a model' }).click();
    await expect(page).toHaveURL(/\/jobs\/new/);
    await page.getByRole('link', { name: 'Jobs' }).click();
    await expect(page).toHaveURL(/\/jobs$/);
    await page.getByRole('link', { name: 'Algorithms' }).click();
    await expect(page).toHaveURL(/\/algorithms$/);
    await page.getByRole('link', { name: 'llm' }).click();
    await expect(page).toHaveURL('/');
  });
});

test.describe('Demo Scene 4-6: New Job Wizard (Classification)', () => {
  test('walks through dataset upload, mapping, and model configuration', async ({ page }) => {
    await page.goto('/jobs/new');
    await expect(page.locator('h2')).toContainText('1. Dataset');

    // Scene 4: CSV upload
    await uploadFile(page, CLASSIFICATION_CSV, 'demo-dataset.csv');
    const uploadResponse = await page.waitForResponse(
      (res) => res.url().includes('/api/v1/datasets') && res.status() === 201,
    );
    const uploadData = await uploadResponse.json();
    const datasetId = uploadData.datasetId;
    await expect(page.locator('h2')).toContainText('2. Mapping');

    // Scene 5: Column mapping
    await page.getByRole('checkbox', { name: 'feature1' }).check();
    await page.getByRole('checkbox', { name: 'feature2' }).check();
    await page.getByRole('checkbox', { name: 'feature3' }).check();
    await page.locator('#task-type').selectOption('classification');
    await page.locator('#label-column').selectOption('label');

    await page.getByRole('button', { name: 'Continue to model' }).click();
    await expect(page.locator('h2')).toContainText('3. Model');

    // Scene 6: Model configuration - create job via API (algorithm UI loads from backend)
    const jobId = await createJobViaAPI(
      page,
      datasetId,
      'classification',
      'random_forest_classifier',
    );
    await page.goto(`/jobs/${jobId}`);
    await expect(page.locator('h1')).toContainText('Job');
  });
});

test.describe('Demo Scene 7: Job Lifecycle & Polling', () => {
  test('shows job status progression and artefact download', async ({ page }) => {
    await page.goto('/jobs/new');
    await uploadFile(page, CLASSIFICATION_CSV, 'demo-dataset.csv');
    const uploadResponse = await page.waitForResponse(
      (res) => res.url().includes('/api/v1/datasets') && res.status() === 201,
    );
    const uploadData = await uploadResponse.json();
    const datasetId = uploadData.datasetId;

    await page.getByRole('checkbox', { name: 'feature1' }).check();
    await page.getByRole('checkbox', { name: 'feature2' }).check();
    await page.getByRole('checkbox', { name: 'feature3' }).check();
    await page.locator('#task-type').selectOption('classification');
    await page.locator('#label-column').selectOption('label');
    await page.getByRole('button', { name: 'Continue to model' }).click();

    // Create job via API
    const jobId = await createJobViaAPI(
      page,
      datasetId,
      'classification',
      'random_forest_classifier',
    );

    // Navigate to job status page
    await page.goto(`/jobs/${jobId}`);
    await expect(page.locator('h1')).toContainText('Job');

    // Wait for completion via backend API polling
    const job = await waitForJobCompletion(page, jobId);
    expect(job.status).toBe('completed');

    // Verify artefacts via backend API
    const artefacts = await verifyArtefacts(page, jobId);
    expect(artefacts.metrics?.accuracy).toBeGreaterThan(0);

    // Navigate to jobs list to show the completed job
    await page.goto('/jobs');
    await expect(page.locator('h1')).toContainText('Jobs');
  });
});

test.describe('Demo Scene 8: Jobs History', () => {
  test('lists jobs and opens a job detail', async ({ page }) => {
    await page.goto('/jobs');
    await expect(page.locator('h1')).toContainText('Jobs');
    await page.getByRole('link', { name: 'New job' }).click();
    await page.waitForURL(/\/jobs\/new/);
  });
});

test.describe('Demo Scene 9: Error Handling', () => {
  test('shows 404 for missing job', async ({ page }) => {
    await page.goto('/jobs/00000000-0000-0000-0000-000000000000');
    await expect(page.locator('h1')).toContainText('Job status');
    // The catalogue turns the 404 into a title, a guidance line, and an action.
    // The route announcer also carries `role="alert"`, so filter it out.
    const alert = page.getByRole('alert').filter({ hasText: 'Job not found' });
    await expect(alert).toContainText('Job not found', { timeout: 15000 });
    await expect(alert).toContainText('The job was deleted.');
    await expect(page.getByRole('link', { name: 'Back to jobs' })).toBeVisible();
  });
});

test.describe('Demo Scene 11: Regression Workflow', () => {
  test('end-to-end regression training', async ({ page }) => {
    await page.goto('/jobs/new');
    await uploadFile(page, REGRESSION_CSV, 'demo-regression.csv');
    const uploadResponse = await page.waitForResponse(
      (res) => res.url().includes('/api/v1/datasets') && res.status() === 201,
    );
    const uploadData = await uploadResponse.json();
    const datasetId = uploadData.datasetId;

    await page.getByRole('checkbox', { name: 'sqft' }).check();
    await page.getByRole('checkbox', { name: 'bedrooms' }).check();
    await page.getByRole('checkbox', { name: 'age' }).check();
    await page.locator('#task-type').selectOption('regression');
    await page.locator('#label-column').selectOption('price');
    await page.getByRole('button', { name: 'Continue to model' }).click();

    // Create job via API
    const jobId = await createJobViaAPI(page, datasetId, 'regression', 'linear_regression');

    // Navigate to job status page
    await page.goto(`/jobs/${jobId}`);
    await expect(page.locator('h1')).toContainText('Job');

    // Wait for completion via backend API polling
    const job = await waitForJobCompletion(page, jobId);
    expect(job.status).toBe('completed');

    // Verify regression metrics via backend API
    expect(job.metrics?.rmse).toBeGreaterThan(0);
    expect(job.metrics?.mae).toBeGreaterThan(0);
    expect(job.metrics?.r2).toBeGreaterThanOrEqual(0);

    // The metrics list spells out what each score means
    await expect(page.getByText('Root mean squared error')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Mean absolute error')).toBeVisible();
    await expect(page.getByText('Coefficient of determination')).toBeVisible();

    // Verify artefacts
    await verifyArtefacts(page, jobId);
  });
});

test.describe('Demo Scene 12: Semi-supervised Workflow', () => {
  test('trains self-training on a partially labelled CSV', async ({ page }) => {
    await page.goto('/jobs/new');
    await uploadFile(page, SEMI_SUPERVISED_CSV, 'semi-supervised.csv');
    const uploadResponse = await page.waitForResponse(
      (res) => res.url().includes('/api/v1/datasets') && res.status() === 201,
    );
    const uploadData = await uploadResponse.json();
    const datasetId = uploadData.datasetId;

    await page.getByRole('checkbox', { name: 'feature1' }).check();
    await page.getByRole('checkbox', { name: 'feature2' }).check();
    await page.getByRole('checkbox', { name: 'feature3' }).check();
    await page.locator('#task-type').selectOption('semi_supervised');
    await page.locator('#label-column').selectOption('label');

    // The mapping step reports the labelled/unlabelled split
    await expect(page.getByText(/labelled ·/)).toBeVisible();

    await page.getByRole('button', { name: 'Continue to model' }).click();
    await expect(page.locator('h2')).toContainText('3. Model');

    const jobId = await createJobViaAPI(
      page,
      datasetId,
      'semi_supervised',
      'self_training_classifier',
    );
    await page.goto(`/jobs/${jobId}`);
    const job = await waitForJobCompletion(page, jobId);
    expect(job.status).toBe('completed');
    expect(job.metrics?.labelledRows).toBe(12);
    expect(job.metrics?.unlabelledRows).toBe(4);
    expect(job.metrics?.iterations).toBeGreaterThanOrEqual(1);
    expect(job.metrics?.accuracy).toBeGreaterThan(0);

    // Metrics spell out what the counts mean
    await expect(page.getByText(/^Labelled rows/)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/^Self-training iterations/)).toBeVisible();

    await verifyArtefacts(page, jobId);
  });
});

test.describe('Demo Scene 13: Unsupervised Workflows', () => {
  test('clusters and detects anomalies without a label column', async ({ page }) => {
    await page.goto('/jobs/new');
    await uploadFile(page, CLASSIFICATION_CSV, 'demo-dataset.csv');
    const uploadResponse = await page.waitForResponse(
      (res) => res.url().includes('/api/v1/datasets') && res.status() === 201,
    );
    const uploadData = await uploadResponse.json();
    const datasetId = uploadData.datasetId;

    await page.getByRole('checkbox', { name: 'feature1' }).check();
    await page.getByRole('checkbox', { name: 'feature2' }).check();
    await page.getByRole('checkbox', { name: 'feature3' }).check();
    await page.locator('#task-type').selectOption('clustering');
    // Unsupervised tasks never offer a label column
    await expect(page.locator('#label-column')).toHaveCount(0);

    await page.getByRole('button', { name: 'Continue to model' }).click();
    await expect(page.locator('h2')).toContainText('3. Model');

    const clusterJobId = await createJobViaAPI(page, datasetId, 'clustering', 'kmeans');
    await page.goto(`/jobs/${clusterJobId}`);
    const clusterJob = await waitForJobCompletion(page, clusterJobId);
    expect(clusterJob.status).toBe('completed');
    expect(clusterJob.metrics?.silhouette).not.toBeNull();
    expect(clusterJob.metrics?.inertia).toBeGreaterThan(0);
    await expect(page.getByRole('heading', { name: 'Silhouette Plot' })).toBeVisible({
      timeout: 10000,
    });
    await verifyArtefacts(page, clusterJobId);

    const anomalyJobId = await createJobViaAPI(
      page,
      datasetId,
      'anomaly_detection',
      'isolation_forest',
    );
    await page.goto(`/jobs/${anomalyJobId}`);
    const anomalyJob = await waitForJobCompletion(page, anomalyJobId);
    expect(anomalyJob.status).toBe('completed');
    expect(anomalyJob.metrics?.anomalyCount).toBeGreaterThanOrEqual(0);
    expect(anomalyJob.metrics?.anomalyRate).toBeGreaterThanOrEqual(0);
    expect(anomalyJob.metrics?.anomalyRate).toBeLessThanOrEqual(1);
    await expect(page.getByText('Anomaly rate')).toBeVisible({ timeout: 10000 });
    await verifyArtefacts(page, anomalyJobId);
  });
});

test.describe('Demo Scene 14: Dark Theme Walkthrough', () => {
  test('renders key flows in dark theme', async ({ page }) => {
    await page.goto('/');
    const toggle = page.getByLabel('Toggle theme');
    await toggle.click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'llm-dark');
    await expect(page.locator('h1')).toContainText('llm-client');
    await page.getByRole('link', { name: 'Train a model' }).click();
    await expect(page.locator('h1')).toContainText('New job', { timeout: 10000 });
    await toggle.click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'llm-light');
  });
});
