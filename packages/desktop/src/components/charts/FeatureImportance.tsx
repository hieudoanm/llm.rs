'use client';

import { BarChart } from './BarChart';

interface FeatureImportanceProps {
  title: string;
  caption: string;
  data: { feature: string[]; importance: number[]; method: string } | null | undefined;
}

export const FeatureImportance = ({ title, caption, data }: FeatureImportanceProps) => {
  if (!data || !data.feature || data.feature.length === 0) {
    return null;
  }

  return (
    <BarChart
      title={title}
      caption={`${caption} (method: ${data.method})`}
      data={data.feature.map((feature, i) => ({
        label: feature,
        value: data.importance[i],
      }))}
      xLabel='Feature'
      yLabel='Importance'
    />
  );
};
