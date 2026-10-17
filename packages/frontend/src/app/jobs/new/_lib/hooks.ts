import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useRouter } from 'next/navigation';
import { apiGet, apiPost, apiUpload } from '@/lib/api';
import { QUERY_CACHE } from '@/lib/queryCache';
import { TaskType, WizardStep, ExportFormat, Dataset, Algorithm, JobAccepted } from './types';
import { LABEL_TASKS, RL_TASK, HYPERPARAMETER_FIELDS } from './constants';
import { useRlColumns } from './useRlColumns';
import {
  clearWizardState,
  readWizardState,
  writeWizardState,
  type WizardState,
} from './persistence';
import { validateWizard, type ValidationIssue } from './validation';

/** True when the backend reports that the dataset no longer exists. */
const isDatasetNotFound = (error: unknown): boolean => {
  const candidate = error as { status?: number; serverCode?: string; code?: string } | null;
  if (!candidate) return false;
  return (
    candidate.status === 404 &&
    (candidate.serverCode === 'DATASET_NOT_FOUND' || candidate.code === 'DATASET_NOT_FOUND')
  );
};

/** The dataset of a restored session. */
const datasetOf = (stored: WizardState): Dataset => ({
  datasetId: stored.datasetId,
  sha256: '',
  filename: stored.filename,
  byteSize: 0,
  columns: stored.columns,
  rowCount: stored.rowCount,
  createdAt: '',
});

export const useNewJobWizard = () => {
  const router = useRouter();
  // The stored session is read once, during the initial render, so the wizard
  // never has to restore it with a cascading setState in an effect.
  const [stored] = useState<WizardState | null>(() => readWizardState());
  const [step, setStep] = useState<WizardStep>(() => (stored ? 'mapping' : 'dataset'));
  const [dataset, setDataset] = useState<Dataset | null>(() => (stored ? datasetOf(stored) : null));
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<{ name: string; size: number } | null>(null);
  const [featureColumns, setFeatureColumns] = useState<string[]>(
    () => stored?.featureColumns ?? [],
  );
  const [taskType, setTaskType] = useState<TaskType>(() => stored?.taskType ?? 'classification');
  const [labelColumn, setLabelColumn] = useState<string>(() => stored?.labelColumn ?? '');
  const [algorithmId, setAlgorithmId] = useState<string>(() => stored?.algorithmId ?? '');
  const [hyperparameters, setHyperparameters] = useState<Record<string, string | number | boolean>>(
    () => stored?.hyperparameters ?? {},
  );
  const [contaminationMode, setContaminationMode] = useState<'auto' | 'custom'>('auto');
  const [exportPkl, setExportPkl] = useState(() => stored?.exportPkl ?? false);
  const [exportJoblib, setExportJoblib] = useState(() => stored?.exportJoblib ?? false);
  const [restored, setRestored] = useState(() => stored !== null);
  const [notice, setNotice] = useState<string | null>(null);
  const idempotencyKeyRef = useRef('');

  const handleLabelChange = (value: string) => setLabelColumn(value);

  const requiresLabel = LABEL_TASKS.has(taskType);
  const isReinforcement = taskType === RL_TASK;
  const rl = useRlColumns(featureColumns);
  const { rlColumns, rlReady, resetRlColumns } = rl;

  // --- persist the current session -------------------------------------------
  useEffect(() => {
    if (!dataset) return;
    const state: WizardState = {
      datasetId: dataset.datasetId,
      filename: dataset.filename,
      rowCount: dataset.rowCount,
      columns: dataset.columns,
      featureColumns,
      taskType,
      labelColumn,
      algorithmId,
      hyperparameters,
      exportPkl,
      exportJoblib,
      rlColumns: rlColumns as WizardState['rlColumns'],
    };
    writeWizardState(state);
  }, [
    dataset,
    featureColumns,
    taskType,
    labelColumn,
    algorithmId,
    hyperparameters,
    exportPkl,
    exportJoblib,
    rlColumns,
  ]);

  const uploadMutation = useMutation({
    mutationFn: (formData: FormData) => apiUpload<Dataset>('/api/v1/datasets', formData),
    onSuccess: (data) => {
      setDataset(data);
      setUploadError(null);
      setRestored(false);
      setNotice(null);
      setStep('mapping');
    },
    onError: (error: Error) => setUploadError(error.message),
  });

  const { data: algorithmsData, isLoading: loadingAlgorithms } = useQuery({
    queryKey: ['algorithms', taskType],
    queryFn: () => apiGet<{ algorithms: Algorithm[] }>(`/api/v1/algorithms?taskType=${taskType}`),
    select: (data) => data.algorithms,
    ...QUERY_CACHE.catalogue,
  });

  const algorithms = algorithmsData ?? [];
  const selectedAlgorithm = algorithms.find((a) => a.algorithmId === algorithmId);
  const availableExports = selectedAlgorithm?.exports ?? ['pkl', 'joblib'];
  const hyperparameterFields = useMemo(
    () => (algorithmId ? (HYPERPARAMETER_FIELDS[algorithmId] ?? []) : []),
    [algorithmId],
  );

  const setExport: Record<ExportFormat, (checked: boolean) => void> = {
    pkl: setExportPkl,
    joblib: setExportJoblib,
  };
  const exportState: Record<ExportFormat, boolean> = {
    pkl: exportPkl,
    joblib: exportJoblib,
  };

  const handleTaskChange = (next: TaskType) => {
    setTaskType(next);
    setAlgorithmId('');
    setHyperparameters({});
    setContaminationMode('auto');
    if (next !== RL_TASK) {
      resetRlColumns();
    }
  };

  const handleAlgorithmChange = (next: string) => {
    setAlgorithmId(next);
    setHyperparameters({});
    setContaminationMode('auto');
    if (next === 'isolation_forest') {
      setHyperparameters((current) => ({ ...current, contamination: 'auto' }));
    }
  };

  const setHyperparameter = useCallback((key: string, value: string | number | boolean) => {
    setHyperparameters((current) => ({ ...current, [key]: value }));
  }, []);

  const labelledRows = (() => {
    if (!dataset || !labelColumn) return 0;
    return dataset.rowCount;
  })();

  const unlabelledRows = (() => {
    if (taskType !== 'semi_supervised') return 0;
    return 0;
  })();

  // --- live validation --------------------------------------------------------
  const validation = useMemo(
    () =>
      validateWizard({
        filename: selectedFile?.name ?? dataset?.filename ?? null,
        fileSize: selectedFile?.size ?? (dataset ? dataset.byteSize : null),
        uploaded: Boolean(dataset),
        columns: (dataset?.columns ?? []).map((column) => ({
          name: column.name,
          inferredDtype: column.inferredDtype,
        })),
        rowCount: dataset?.rowCount ?? 0,
        featureColumns,
        labelColumn,
        taskType,
        algorithmId,
        hyperparameters,
        hyperparameterFields,
      }),
    [
      selectedFile,
      dataset,
      featureColumns,
      labelColumn,
      taskType,
      algorithmId,
      hyperparameters,
      hyperparameterFields,
    ],
  );

  const issues: ValidationIssue[] = [
    ...validation.dataset,
    ...validation.mapping,
    ...validation.model,
  ];

  /** The issue of one control, when there is one. */
  const issueFor = (field: string): ValidationIssue | undefined =>
    issues.find((issue) => issue.field === field);

  const mappingReady = Boolean(
    dataset &&
    featureColumns.length > 0 &&
    (!requiresLabel || labelColumn) &&
    (!isReinforcement || rlReady),
  );

  const canStartTraining = Boolean(
    dataset &&
    featureColumns.length > 0 &&
    algorithmId &&
    (!requiresLabel || labelColumn) &&
    (!isReinforcement || rlReady),
  );

  const createJobMutation = useMutation<JobAccepted, Error, void>({
    mutationFn: () => {
      if (!dataset) throw new Error('Missing dataset');
      const body: Record<string, unknown> = {
        datasetId: dataset.datasetId,
        taskType,
        algorithmId,
        featureColumns,
      };
      if (requiresLabel) body.labelColumn = labelColumn;
      if (isReinforcement) body.rlColumns = rlColumns;
      const hyper: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(hyperparameters)) {
        if (value === '' || value === null || value === undefined) continue;
        hyper[key] = value;
      }
      if (Object.keys(hyper).length > 0) body.hyperparameters = hyper;
      body.exportPkl = exportPkl;
      body.exportJoblib = exportJoblib;
      // One key per job configuration: retries reuse it (so a double-submit
      // cannot start two jobs) while a new job after success gets a fresh key.
      if (!idempotencyKeyRef.current) idempotencyKeyRef.current = crypto.randomUUID();
      return apiPost<JobAccepted>('/api/v1/jobs', body, {
        headers: { 'Idempotency-Key': idempotencyKeyRef.current },
      });
    },
    onSuccess: (data) => {
      idempotencyKeyRef.current = '';
      clearWizardState();
      router.push(`/jobs/${data.jobId}`);
    },
    onError: (error: unknown) => {
      if (!isDatasetNotFound(error)) return;
      // The dataset was removed on the server: drop it and restart the wizard
      // at the upload step instead of failing with a raw 404.
      clearWizardState();
      setDataset(null);
      setFeatureColumns([]);
      setLabelColumn('');
      setAlgorithmId('');
      setHyperparameters({});
      resetRlColumns();
      setRestored(false);
      setNotice('The uploaded dataset no longer exists. Upload the CSV again to continue.');
      setStep('dataset');
    },
  });

  const handleUpload = (file: File) => {
    setUploadError(null);
    setNotice(null);
    setSelectedFile({ name: file.name, size: file.size });
    const formData = new FormData();
    formData.append('file', file);
    uploadMutation.mutate(formData);
  };

  const toggleFeature = (name: string) => {
    setFeatureColumns((current) =>
      current.includes(name) ? current.filter((c) => c !== name) : [...current, name],
    );
  };

  /** Forget the stored session and start over from the upload step. */
  const startOver = useCallback(() => {
    clearWizardState();
    setDataset(null);
    setSelectedFile(null);
    setFeatureColumns([]);
    setLabelColumn('');
    setAlgorithmId('');
    setHyperparameters({});
    setContaminationMode('auto');
    setExportPkl(false);
    setExportJoblib(false);
    resetRlColumns();
    setRestored(false);
    setNotice(null);
    setStep('dataset');
  }, [resetRlColumns]);

  return {
    step,
    setStep,
    dataset,
    setDataset,
    uploadError,
    selectedFile,
    featureColumns,
    setFeatureColumns,
    taskType,
    labelColumn,
    setLabelColumn,
    onLabelChange: handleLabelChange,
    isReinforcement,
    ...rl,
    algorithmId,
    hyperparameters,
    contaminationMode,
    setContaminationMode,
    exportState,
    setExport,
    requiresLabel,
    uploadMutation,
    algorithms,
    loadingAlgorithms,
    selectedAlgorithm,
    availableExports,
    hyperparameterFields,
    handleTaskChange,
    handleAlgorithmChange,
    setHyperparameter,
    labelledRows,
    unlabelledRows,
    mappingReady,
    canStartTraining,
    createJobMutation,
    handleUpload,
    toggleFeature,
    validation,
    issues,
    issueFor,
    restored,
    notice,
    setNotice,
    startOver,
  };
};
