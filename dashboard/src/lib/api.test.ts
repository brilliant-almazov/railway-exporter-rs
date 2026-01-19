import { describe, it, expect, vi } from 'vitest';
import { mapApiToMetrics, fetchMetrics } from './api';
import type { ApiMetricsJson } from '@/types';

// =============================================================================
// Mock Data
// =============================================================================

const mockApiResponse: ApiMetricsJson = {
  project: {
    name: 'test-project',
    current_usage_usd: 25.50,
    estimated_monthly_usd: 75.00,
    daily_average_usd: 2.50,
    days_elapsed: 10,
    days_remaining: 20,
  },
  services: [
    {
      id: 'svc-1',
      name: 'api-server',
      icon: 'data:image/png;base64,abc123',
      group: 'backend',
      cpu_usage: 1440, // 1440 vCPU-minutes
      memory_usage: 720, // 720 GB-minutes
      disk_usage: 100,
      network_tx: 0.5,
      cost_usd: 15.00,
      estimated_monthly_usd: 45.00,
      isDeleted: false,
    },
    {
      id: 'svc-2',
      name: 'web-frontend',
      icon: '',
      group: 'frontend',
      cpu_usage: 720,
      memory_usage: 360,
      disk_usage: 50,
      network_tx: 0.1,
      cost_usd: 10.50,
      estimated_monthly_usd: 30.00,
      isDeleted: false,
    },
    {
      id: 'svc-deleted',
      name: 'old-service',
      icon: '',
      group: '',
      cpu_usage: 0,
      memory_usage: 0,
      disk_usage: 0,
      network_tx: 0,
      cost_usd: 0,
      estimated_monthly_usd: 0,
      isDeleted: true,
    },
  ],
  scrape_timestamp: 1700000000,
  scrape_duration_seconds: 0.15,
};

// =============================================================================
// mapApiToMetrics Tests
// =============================================================================

describe('mapApiToMetrics', () => {
  it('maps project summary correctly', () => {
    const result = mapApiToMetrics(mockApiResponse);

    expect(result.project).toBe('test-project');
    expect(result.currentUsage).toBe(25.50);
    expect(result.estimatedMonthly).toBe(75.00);
    expect(result.dailyAverage).toBe(2.50);
    expect(result.daysInPeriod).toBe(10);
    expect(result.daysRemaining).toBe(20);
  });

  it('maps scrape metadata correctly', () => {
    const result = mapApiToMetrics(mockApiResponse);

    expect(result.scrapeSuccess).toBe(1);
    expect(result.scrapeDuration).toBe(0.15);
  });

  it('maps services with correct fields', () => {
    const result = mapApiToMetrics(mockApiResponse);

    // Services are sorted by cost (descending)
    expect(result.services).toHaveLength(3);
    expect(result.services[0].name).toBe('api-server'); // $15
    expect(result.services[1].name).toBe('web-frontend'); // $10.50
    expect(result.services[2].name).toBe('old-service'); // $0
  });

  it('calculates average metrics correctly', () => {
    const result = mapApiToMetrics(mockApiResponse);
    const apiService = result.services[0]; // api-server

    // 10 days * 24 hours * 60 minutes = 14400 minutes
    const minutesInPeriod = 10 * 24 * 60;

    expect(apiService.avgCpu).toBeCloseTo(1440 / minutesInPeriod);
    expect(apiService.avgMemory).toBeCloseTo(720 / minutesInPeriod);
    expect(apiService.avgDisk).toBeCloseTo(100 / minutesInPeriod);
  });

  it('handles empty group as ungrouped', () => {
    const result = mapApiToMetrics(mockApiResponse);
    const deletedService = result.services.find(s => s.name === 'old-service');

    expect(deletedService?.group).toBe('ungrouped');
  });

  it('preserves isDeleted flag from server', () => {
    const result = mapApiToMetrics(mockApiResponse);

    const activeService = result.services.find(s => s.name === 'api-server');
    const deletedService = result.services.find(s => s.name === 'old-service');

    expect(activeService?.isDeleted).toBe(false);
    expect(deletedService?.isDeleted).toBe(true);
  });

  it('handles zero days elapsed (avoid division by zero)', () => {
    const zeroDaysResponse: ApiMetricsJson = {
      ...mockApiResponse,
      project: {
        ...mockApiResponse.project,
        days_elapsed: 0,
      },
    };

    const result = mapApiToMetrics(zeroDaysResponse);

    expect(result.services[0].avgCpu).toBe(0);
    expect(result.services[0].avgMemory).toBe(0);
    expect(result.services[0].avgDisk).toBe(0);
  });

  it('handles empty services array', () => {
    const emptyServicesResponse: ApiMetricsJson = {
      ...mockApiResponse,
      services: [],
    };

    const result = mapApiToMetrics(emptyServicesResponse);

    expect(result.services).toHaveLength(0);
  });
});

// =============================================================================
// fetchMetrics Tests
// =============================================================================

describe('fetchMetrics', () => {
  it('fetches and transforms metrics from API', async () => {
    // Mock fetch
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockApiResponse),
    });

    const result = await fetchMetrics('http://localhost:9090/metrics');

    expect(fetch).toHaveBeenCalledWith('http://localhost:9090/metrics', {
      headers: { Accept: 'application/json' },
    });
    expect(result.project).toBe('test-project');
    expect(result.services).toHaveLength(3);
  });

  it('throws error on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 500,
    });

    await expect(fetchMetrics('http://localhost:9090/metrics')).rejects.toThrow(
      'HTTP 500'
    );
  });

  it('throws error on network failure', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

    await expect(fetchMetrics('http://localhost:9090/metrics')).rejects.toThrow(
      'Network error'
    );
  });
});
