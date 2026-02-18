import { describe, it, expect } from 'vitest';
import { StaleGuard } from './stale-guard';

describe('StaleGuard', () => {
  it('creates a guard with initial version 0', () => {
    const guard = new StaleGuard();
    expect(guard.version).toBe(0);
  });

  describe('begin()', () => {
    it('increments version and returns the new version', () => {
      const guard = new StaleGuard();
      const v1 = guard.begin();
      expect(v1).toBe(1);
      const v2 = guard.begin();
      expect(v2).toBe(2);
    });
  });

  describe('isStale()', () => {
    it('returns false when version matches (no interleaved call)', () => {
      const guard = new StaleGuard();
      const token = guard.begin();
      expect(guard.isStale(token)).toBe(false);
    });

    it('returns true when a newer call has begun', () => {
      const guard = new StaleGuard();
      const token1 = guard.begin();
      guard.begin(); // token2 — supersedes token1
      expect(guard.isStale(token1)).toBe(true);
    });

    it('returns false for the latest token even after multiple begins', () => {
      const guard = new StaleGuard();
      guard.begin();
      guard.begin();
      const token3 = guard.begin();
      expect(guard.isStale(token3)).toBe(false);
    });
  });

  describe('reset()', () => {
    it('resets version to 0', () => {
      const guard = new StaleGuard();
      guard.begin();
      guard.begin();
      guard.reset();
      expect(guard.version).toBe(0);
    });

    it('makes all previous tokens stale after reset', () => {
      const guard = new StaleGuard();
      const token = guard.begin();
      guard.reset();
      expect(guard.isStale(token)).toBe(true);
    });
  });

  describe('real-world race condition simulation', () => {
    it('prevents stale write when fast project switch occurs', async () => {
      const guard = new StaleGuard();
      let result: string | null = null;

      // Simulate: open project A (slow), then immediately open project B (fast)
      const tokenA = guard.begin();
      const tokenB = guard.begin();

      // Project B resolves first
      if (!guard.isStale(tokenB)) {
        result = 'Project B';
      }

      // Project A resolves later (stale!)
      if (!guard.isStale(tokenA)) {
        result = 'Project A'; // This should NOT execute
      }

      expect(result).toBe('Project B');
    });

    it('allows write when no interleaved call occurs', async () => {
      const guard = new StaleGuard();
      let result: string | null = null;

      const token = guard.begin();
      // Simulate async work
      await new Promise((r) => setTimeout(r, 1));

      if (!guard.isStale(token)) {
        result = 'Success';
      }

      expect(result).toBe('Success');
    });
  });
});
