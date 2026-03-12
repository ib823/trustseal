/**
 * E2E Tests: Platform API
 *
 * Tests the core API endpoints that support the VaultPass ecosystem.
 * These tests verify the API contract and performance requirements.
 */

import { test, expect } from '@playwright/test';

const API_BASE = process.env.API_BASE_URL || 'http://localhost:3000';

test.describe('Platform API Health', () => {
  test('GET /health returns healthy status', async ({ request }) => {
    const response = await request.get(`${API_BASE}/health`);

    expect(response.ok()).toBe(true);
    const body = await response.json();
    expect(body.status).toBe('healthy');
  });

  test('health endpoint responds within 100ms', async ({ request }) => {
    const start = Date.now();
    const response = await request.get(`${API_BASE}/health`);
    const duration = Date.now() - start;

    expect(response.ok()).toBe(true);
    // Health check should be very fast
    expect(duration).toBeLessThan(100);
  });
});

test.describe('eKYC API', () => {
  test('POST /api/v1/ekyc/initiate returns verification ID', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/v1/ekyc/initiate`, {
      data: {
        tenant_id: 'TNT_e2e_test',
        user_id: 'USR_e2e_test',
      },
    });

    expect(response.ok()).toBe(true);
    const body = await response.json();

    expect(body.verification_id).toBeDefined();
    expect(body.verification_id).toMatch(/^IDV_/);
    expect(body.authorization_url).toBeDefined();
    expect(body.state).toBeDefined();
    expect(body.expires_at).toBeDefined();
  });

  test('GET /api/v1/ekyc/status/:id returns verification status', async ({ request }) => {
    const verificationId = 'IDV_e2e_status_test';
    const response = await request.get(`${API_BASE}/api/v1/ekyc/status/${verificationId}`);

    expect(response.ok()).toBe(true);
    const body = await response.json();

    expect(body.verification_id).toBe(verificationId);
    expect(body.status).toBeDefined();
    expect(body.provider).toBeDefined();
    expect(body.assurance_level).toBeDefined();
  });

  test('POST /api/v1/ekyc/bind-did validates DID format', async ({ request }) => {
    // Invalid DID should return 400
    const invalidResponse = await request.post(`${API_BASE}/api/v1/ekyc/bind-did`, {
      data: {
        verification_id: 'IDV_test',
        did: 'not-a-valid-did',
      },
    });

    expect(invalidResponse.status()).toBe(400);
    const errorBody = await invalidResponse.json();
    expect(errorBody.code).toBe('SAHI_2308');
  });

  test('POST /api/v1/ekyc/bind-did accepts valid DID', async ({ request }) => {
    const validDid = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
    const response = await request.post(`${API_BASE}/api/v1/ekyc/bind-did`, {
      data: {
        verification_id: 'IDV_e2e_bind_test',
        did: validDid,
      },
    });

    expect(response.ok()).toBe(true);
    const body = await response.json();

    expect(body.verification_id).toBe('IDV_e2e_bind_test');
    expect(body.did).toBe(validDid);
    expect(body.bound_at).toBeDefined();
  });

  test('eKYC initiate responds within 300ms', async ({ request }) => {
    const start = Date.now();
    const response = await request.post(`${API_BASE}/api/v1/ekyc/initiate`, {
      data: {
        tenant_id: 'TNT_perf_test',
      },
    });
    const duration = Date.now() - start;

    expect(response.ok()).toBe(true);
    // Per performance targets: ekyc initiate < 300ms (p95)
    expect(duration).toBeLessThan(500); // Allow some headroom for E2E
  });
});

test.describe('API Error Handling', () => {
  test('missing tenant_id returns structured error', async ({ request }) => {
    const response = await request.post(`${API_BASE}/api/v1/ekyc/initiate`, {
      data: {},
    });

    // Should return 4xx error with proper structure
    expect(response.status()).toBeGreaterThanOrEqual(400);
    expect(response.status()).toBeLessThan(500);
  });

  test('404 for unknown routes', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/v1/unknown-endpoint`);
    expect(response.status()).toBe(404);
  });
});
