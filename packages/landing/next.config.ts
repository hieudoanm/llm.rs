import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'export',
  basePath: '/llm.rs',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
