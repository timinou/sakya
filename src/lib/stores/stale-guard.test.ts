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

    it('invalidates all previous tokens', () => {
      const guard = new StaleGuard();
      const token1 = guard.begin();
      guard.begin(); // supersedes token1
      expect(guard.isStale(token1)).toBe(true);
    });
  });

  describe('snapshot()', () => {
    it('returns current version without incrementing', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      const snap = guard.snapshot();
      expect(snap).toBe(1);
      expect(guard.version).toBe(1); // unchanged
    });

    it('multiple snapshots return the same version', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      const snap1 = guard.snapshot();
      const snap2 = guard.snapshot();
      expect(snap1).toBe(snap2);
      expect(guard.version).toBe(1); // unchanged
    });

    it('snapshots are not stale when no reset or begin has occurred', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      const snap = guard.snapshot();
      expect(guard.isStale(snap)).toBe(false);
    });

    it('concurrent snapshots do NOT invalidate each other', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      const snap1 = guard.snapshot();
      const snap2 = guard.snapshot();
      const snap3 = guard.snapshot();
      // All snapshots captured the same version — none are stale
      expect(guard.isStale(snap1)).toBe(false);
      expect(guard.isStale(snap2)).toBe(false);
      expect(guard.isStale(snap3)).toBe(false);
    });

    it('snapshots become stale after reset()', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      const snap = guard.snapshot();
      guard.reset(); // version = 2
      expect(guard.isStale(snap)).toBe(true);
    });

    it('snapshots become stale after begin()', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      const snap = guard.snapshot();
      guard.begin(); // version = 2
      expect(guard.isStale(snap)).toBe(true);
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
    it('increments version (monotonically increases)', () => {
      const guard = new StaleGuard();
      guard.begin(); // version = 1
      guard.begin(); // version = 2
      guard.reset(); // version = 3
      expect(guard.version).toBe(3);
    });

    it('makes all previous tokens stale after reset', () => {
      const guard = new StaleGuard();
      const token = guard.begin();
      guard.reset();
      expect(guard.isStale(token)).toBe(true);
    });

    it('makes initial snapshot (version 0) stale after reset', () => {
      const guard = new StaleGuard();
      const snap = guard.snapshot(); // version = 0
      guard.reset(); // version = 1
      expect(guard.isStale(snap)).toBe(true);
    });
  });

  describe('real-world race condition simulation', () => {
    it('prevents stale write when fast project switch occurs (begin mode)', async () => {
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

    it('allows concurrent operations with snapshot mode (entity store pattern)', async () => {
      const guard = new StaleGuard();
      guard.reset(); // simulate project open (version = 1)

      // Multiple entity types load concurrently — all capture the same snapshot
      const tokenCharacters = guard.snapshot();
      const tokenPlaces = guard.snapshot();
      const tokenItems = guard.snapshot();

      // All three resolve — none should be stale
      expect(guard.isStale(tokenCharacters)).toBe(false);
      expect(guard.isStale(tokenPlaces)).toBe(false);
      expect(guard.isStale(tokenItems)).toBe(false);
    });

    it('snapshot tokens become stale on project switch (reset)', async () => {
      const guard = new StaleGuard();
      guard.reset(); // project A opened (version = 1)

      const tokenA = guard.snapshot(); // entity load for project A

      guard.reset(); // project B opened (version = 2)

      // Project A's load resolves — should be stale
      expect(guard.isStale(tokenA)).toBe(true);

      // Project B's load — should NOT be stale
      const tokenB = guard.snapshot();
      expect(guard.isStale(tokenB)).toBe(false);
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
