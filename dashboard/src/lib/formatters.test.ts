import { describe, it, expect } from 'vitest';
import {
  formatByType,
  formatCurrency,
  formatCpu,
  formatGb,
  formatNetwork,
  formatMb,
  formatMs,
  formatPercent,
  formatNumber,
  formatInteger,
  formatUptime,
  formatInterval,
  formatRelativeTime,
  FORMAT_CONFIG,
} from './formatters';

// =============================================================================
// FORMAT_CONFIG Tests
// =============================================================================

describe('FORMAT_CONFIG', () => {
  it('has cost config with $ prefix', () => {
    expect(FORMAT_CONFIG.cost.prefix).toBe('$');
    expect(FORMAT_CONFIG.cost.decimals).toBe(2);
  });

  it('has percent config with % suffix', () => {
    expect(FORMAT_CONFIG.percent.suffix).toBe('%');
    expect(FORMAT_CONFIG.percent.decimals).toBe(1);
  });

  it('has gb config with GB suffix', () => {
    expect(FORMAT_CONFIG.gb.suffix).toBe(' GB');
    expect(FORMAT_CONFIG.gb.decimals).toBe(2);
  });

  it('has mb config with MB suffix', () => {
    expect(FORMAT_CONFIG.mb.suffix).toBe(' MB');
    expect(FORMAT_CONFIG.mb.decimals).toBe(2);
  });

  it('has ms config with ms suffix and no decimals', () => {
    expect(FORMAT_CONFIG.ms.suffix).toBe('ms');
    expect(FORMAT_CONFIG.ms.decimals).toBe(0);
  });
});

// =============================================================================
// formatByType Tests
// =============================================================================

describe('formatByType', () => {
  it('formats cost values with $ prefix and 2 decimals', () => {
    expect(formatByType(10.5, 'cost')).toBe('$10.50');
    expect(formatByType(0, 'cost')).toBe('$0.00');
    expect(formatByType(1234.567, 'cost')).toBe('$1,234.57');
  });

  it('formats percent values with % suffix and 1 decimal', () => {
    expect(formatByType(50.5, 'percent')).toBe('50.5%');
    expect(formatByType(100, 'percent')).toBe('100.0%');
    expect(formatByType(0.1, 'percent')).toBe('0.1%');
  });

  it('formats cpu values with 4 decimals', () => {
    expect(formatByType(1.2345, 'cpu')).toBe('1.2345');
    expect(formatByType(0, 'cpu')).toBe('0.0000');
  });

  it('formats gb values with GB suffix and 2 decimals', () => {
    expect(formatByType(1.5, 'gb')).toBe('1.50 GB');
    expect(formatByType(1024, 'gb')).toBe('1,024.00 GB');
  });

  it('formats network values with 4 decimals', () => {
    expect(formatByType(0.1234, 'network')).toBe('0.1234 GB');
  });

  it('formats mb values with MB suffix', () => {
    expect(formatByType(256, 'mb')).toBe('256.00 MB');
  });

  it('formats ms values with no decimals', () => {
    expect(formatByType(150.7, 'ms')).toBe('151ms');
    expect(formatByType(50, 'ms')).toBe('50ms');
  });

  it('formats decimal values with 2 decimals', () => {
    expect(formatByType(3.14159, 'decimal')).toBe('3.14');
  });

  it('formats integer values with 0 decimals', () => {
    expect(formatByType(42.9, 'integer')).toBe('43');
  });
});

// =============================================================================
// Convenience Function Tests
// =============================================================================

describe('formatCurrency', () => {
  it('formats currency values', () => {
    expect(formatCurrency(19.99)).toBe('$19.99');
    expect(formatCurrency(0)).toBe('$0.00');
    expect(formatCurrency(1000)).toBe('$1,000.00');
  });
});

describe('formatCpu', () => {
  it('formats cpu values with 4 decimal places', () => {
    expect(formatCpu(0.5)).toBe('0.5000');
    expect(formatCpu(123.4567)).toBe('123.4567');
  });
});

describe('formatGb', () => {
  it('formats GB values', () => {
    expect(formatGb(2.5)).toBe('2.50 GB');
    expect(formatGb(0)).toBe('0.00 GB');
  });
});

describe('formatNetwork', () => {
  it('formats network values with 4 decimals', () => {
    expect(formatNetwork(0.0001)).toBe('0.0001 GB');
    expect(formatNetwork(10)).toBe('10.0000 GB');
  });
});

describe('formatMb', () => {
  it('formats MB values', () => {
    expect(formatMb(512)).toBe('512.00 MB');
    expect(formatMb(1024)).toBe('1,024.00 MB');
  });
});

describe('formatMs', () => {
  it('formats milliseconds with no decimals', () => {
    expect(formatMs(100)).toBe('100ms');
    expect(formatMs(1500)).toBe('1,500ms');
  });
});

describe('formatPercent', () => {
  it('formats percentage values', () => {
    expect(formatPercent(75)).toBe('75.0%');
    expect(formatPercent(99.9)).toBe('99.9%');
    expect(formatPercent(0)).toBe('0.0%');
  });
});

describe('formatNumber', () => {
  it('formats numbers with specified decimals', () => {
    expect(formatNumber(3.14159, 2)).toBe('3.14');
    expect(formatNumber(3.14159, 4)).toBe('3.1416');
    expect(formatNumber(10, 0)).toBe('10');
  });

  it('defaults to 2 decimals', () => {
    expect(formatNumber(3.14159)).toBe('3.14');
  });
});

describe('formatInteger', () => {
  it('formats as integer with locale separators', () => {
    expect(formatInteger(1234567)).toBe('1,234,567');
    expect(formatInteger(42.9)).toBe('42');
  });
});

// =============================================================================
// Time Formatting Tests
// =============================================================================

describe('formatUptime', () => {
  it('formats seconds as minutes', () => {
    expect(formatUptime(60)).toBe('1m');
    expect(formatUptime(120)).toBe('2m');
    expect(formatUptime(0)).toBe('0m');
  });

  it('formats as hours and minutes', () => {
    expect(formatUptime(3600)).toBe('1h 0m');
    expect(formatUptime(3660)).toBe('1h 1m');
    expect(formatUptime(7200)).toBe('2h 0m');
  });

  it('formats as days and hours', () => {
    expect(formatUptime(86400)).toBe('1d 0h');
    expect(formatUptime(90000)).toBe('1d 1h');
    expect(formatUptime(172800)).toBe('2d 0h');
  });
});

describe('formatInterval', () => {
  it('formats seconds', () => {
    expect(formatInterval(30)).toBe('30s');
    expect(formatInterval(59)).toBe('59s');
  });

  it('formats minutes', () => {
    expect(formatInterval(60)).toBe('1m');
    expect(formatInterval(300)).toBe('5m');
    expect(formatInterval(3599)).toBe('59m');
  });

  it('formats hours', () => {
    expect(formatInterval(3600)).toBe('1h');
    expect(formatInterval(7200)).toBe('2h');
  });
});

describe('formatRelativeTime', () => {
  it('formats seconds ago', () => {
    const now = new Date();
    const date = new Date(now.getTime() - 30000); // 30 seconds ago
    expect(formatRelativeTime(date)).toBe('30s ago');
  });

  it('formats minutes ago', () => {
    const now = new Date();
    const date = new Date(now.getTime() - 120000); // 2 minutes ago
    expect(formatRelativeTime(date)).toBe('2m ago');
  });

  it('formats hours ago', () => {
    const now = new Date();
    const date = new Date(now.getTime() - 7200000); // 2 hours ago
    expect(formatRelativeTime(date)).toBe('2h ago');
  });
});
