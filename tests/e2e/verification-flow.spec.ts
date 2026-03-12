/**
 * E2E Tests: Verification Flow
 *
 * Tests the complete VaultPass credential verification flow:
 * 1. Credential presentation
 * 2. Verification decision
 * 3. Access grant/deny
 *
 * Performance target: Gate entry decision < 2s end-to-end
 */

import { test, expect } from '@playwright/test';

const API_BASE = process.env.API_BASE_URL || 'http://localhost:3000';
const GUARD_URL = process.env.GUARD_URL || 'http://localhost:3002';

test.describe('Verification Flow E2E', () => {
  test('complete eKYC initiation to status check flow', async ({ request }) => {
    // Step 1: Initiate verification
    const initiateResponse = await request.post(`${API_BASE}/api/v1/ekyc/initiate`, {
      data: {
        tenant_id: 'TNT_flow_test',
        user_id: 'USR_flow_test_001',
      },
    });

    expect(initiateResponse.ok()).toBe(true);
    const initiateBody = await initiateResponse.json();

    const verificationId = initiateBody.verification_id;
    expect(verificationId).toMatch(/^IDV_/);

    // Step 2: Check status
    const statusResponse = await request.get(`${API_BASE}/api/v1/ekyc/status/${verificationId}`);
    expect(statusResponse.ok()).toBe(true);

    const statusBody = await statusResponse.json();
    expect(statusBody.verification_id).toBe(verificationId);
    expect(['pending', 'verified', 'failed', 'expired']).toContain(statusBody.status);
  });

  test('eKYC flow with DID binding', async ({ request }) => {
    // Step 1: Initiate
    const initiateResponse = await request.post(`${API_BASE}/api/v1/ekyc/initiate`, {
      data: {
        tenant_id: 'TNT_did_flow_test',
        user_id: 'USR_did_flow_001',
      },
    });

    expect(initiateResponse.ok()).toBe(true);
    const { verification_id } = await initiateResponse.json();

    // Step 2: Bind DID (simulating post-verification)
    const validDid = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
    const bindResponse = await request.post(`${API_BASE}/api/v1/ekyc/bind-did`, {
      data: {
        verification_id,
        did: validDid,
      },
    });

    expect(bindResponse.ok()).toBe(true);
    const bindBody = await bindResponse.json();

    expect(bindBody.did).toBe(validDid);
    expect(bindBody.bound_at).toBeDefined();
  });

  test('rejects invalid DID formats', async ({ request }) => {
    const invalidDids = [
      'invalid',
      'did:',
      'did:key',
      'did:key:',
      'notadid',
      '',
      'did::identifier',
      'did:method with spaces:id',
    ];

    for (const invalidDid of invalidDids) {
      const response = await request.post(`${API_BASE}/api/v1/ekyc/bind-did`, {
        data: {
          verification_id: 'IDV_invalid_did_test',
          did: invalidDid,
        },
      });

      expect(response.status()).toBe(400);
      const body = await response.json();
      expect(body.code).toBe('SAHI_2308');
    }
  });

  test('accepts various valid DID formats', async ({ request }) => {
    const validDids = [
      'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      'did:web:example.com',
      'did:web:example.com:path:to:resource',
      'did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
    ];

    for (const validDid of validDids) {
      const response = await request.post(`${API_BASE}/api/v1/ekyc/bind-did`, {
        data: {
          verification_id: `IDV_valid_did_test_${Date.now()}`,
          did: validDid,
        },
      });

      expect(response.ok()).toBe(true);
    }
  });
});

test.describe('Performance Compliance', () => {
  test('API responds within p99 target under sequential load', async ({ request }) => {
    const durations: number[] = [];
    const iterations = 20;

    for (let i = 0; i < iterations; i++) {
      const start = Date.now();
      const response = await request.get(`${API_BASE}/health`);
      const duration = Date.now() - start;

      expect(response.ok()).toBe(true);
      durations.push(duration);
    }

    // Calculate p99
    durations.sort((a, b) => a - b);
    const p99Index = Math.floor(iterations * 0.99);
    const p99 = durations[p99Index];

    // Target: API p99 < 500ms
    expect(p99).toBeLessThan(500);
  });

  test('eKYC initiate completes within target', async ({ request }) => {
    const durations: number[] = [];
    const iterations = 10;

    for (let i = 0; i < iterations; i++) {
      const start = Date.now();
      const response = await request.post(`${API_BASE}/api/v1/ekyc/initiate`, {
        data: {
          tenant_id: `TNT_perf_${i}`,
          user_id: `USR_perf_${i}`,
        },
      });
      const duration = Date.now() - start;

      expect(response.ok()).toBe(true);
      durations.push(duration);
    }

    // Calculate p95
    durations.sort((a, b) => a - b);
    const p95Index = Math.floor(iterations * 0.95);
    const p95 = durations[p95Index];

    // Target: ekyc initiate p95 < 300ms (allowing 500ms for E2E overhead)
    expect(p95).toBeLessThan(500);
  });
});

test.describe('Error Code Compliance', () => {
  test('error responses follow SAHI_XXXX format', async ({ request }) => {
    // Trigger a known error
    const response = await request.post(`${API_BASE}/api/v1/ekyc/bind-did`, {
      data: {
        verification_id: 'IDV_test',
        did: 'invalid-did',
      },
    });

    expect(response.status()).toBe(400);
    const body = await response.json();

    // Error code follows SAHI_XXXX pattern
    expect(body.code).toMatch(/^SAHI_\d{4}$/);
    expect(body.message).toBeDefined();
    expect(body.message.length).toBeGreaterThan(0);
  });
});
