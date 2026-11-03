import { render, screen } from '@testing-library/react';
import DownloadsPage from '../page';

describe('DownloadsPage', () => {
  it('exports component', () => {
    expect(typeof DownloadsPage).toBe('function');
  });

  it('renders download heading and platform cards', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/get llm\.rs for your platform/i)).toBeInTheDocument();
    expect(screen.getByText('macOS')).toBeInTheDocument();
    expect(screen.getByText('Windows')).toBeInTheDocument();
    expect(screen.getByText('Linux')).toBeInTheDocument();
  });

  it('renders platform descriptions', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/universal \.dmg and \.app\.zip/i)).toBeInTheDocument();
    expect(screen.getByText(/x64 \.msi and \.exe/i)).toBeInTheDocument();
    expect(screen.getByText(/appimage, \.deb, and \.rpm/i)).toBeInTheDocument();
  });

  it('renders download links for each platform', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/download \.dmg/i)).toBeInTheDocument();
    expect(screen.getByText(/download \.msi/i)).toBeInTheDocument();
    expect(screen.getByText(/download appimage/i)).toBeInTheDocument();
  });

  it('renders build-from-source section', () => {
    render(<DownloadsPage />);
    expect(screen.getByText('Build the CLI from source')).toBeInTheDocument();
    expect(screen.getByText(/cargo build --release -p llm-cli/i)).toBeInTheDocument();
  });

  it('renders GitHub link', () => {
    render(<DownloadsPage />);
    expect(screen.getAllByText(/github/i).length).toBeGreaterThanOrEqual(1);
  });
});
