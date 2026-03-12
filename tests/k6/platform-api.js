/**
 * k6 Load Testing for VaultPass Platform API
 *
 * Performance targets (from CLAUDE.md):
 * - API p99 response: < 500ms under 100 concurrent
 *
 * Run with:
 *   k6 run tests/k6/platform-api.js
 *
 * With environment variables:
 *   k6 run -e BASE_URL=http://localhost:3000 tests/k6/platform-api.js
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const healthTrend = new Trend('health_duration');
const ekycInitiateTrend = new Trend('ekyc_initiate_duration');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 20 },   // Ramp up to 20 users
    { duration: '1m', target: 50 },    // Ramp up to 50 users
    { duration: '2m', target: 100 },   // Hold at 100 users (target load)
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    // CRITICAL: p99 < 500ms for all API requests
    http_req_duration: ['p(99)<500'],

    // Health endpoint should be very fast
    'health_duration': ['p(95)<100'],

    // eKYC initiate can be slightly slower (OAuth setup)
    'ekyc_initiate_duration': ['p(95)<300'],

    // Error rate should be < 1%
    'errors': ['rate<0.01'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

/**
 * Setup function - runs once before all VUs start
 */
export function setup() {
  // Verify API is reachable
  const res = http.get(`${BASE_URL}/health`);
  if (res.status !== 200) {
    throw new Error(`API health check failed: ${res.status}`);
  }
  console.log(`API health check passed: ${BASE_URL}`);
  return { baseUrl: BASE_URL };
}

/**
 * Main test function - runs for each VU
 */
export default function (data) {
  const baseUrl = data.baseUrl;

  group('Health Check', () => {
    const start = Date.now();
    const res = http.get(`${baseUrl}/health`);
    healthTrend.add(Date.now() - start);

    const success = check(res, {
      'health status is 200': (r) => r.status === 200,
      'health response has status': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.status === 'healthy';
        } catch {
          return false;
        }
      },
    });
    errorRate.add(!success);
  });

  sleep(0.5);

  group('eKYC Initiate', () => {
    const start = Date.now();
    const payload = JSON.stringify({
      tenant_id: 'TNT_load_test',
      user_id: `USR_${__VU}_${__ITER}`,
    });

    const params = {
      headers: {
        'Content-Type': 'application/json',
      },
    };

    const res = http.post(`${baseUrl}/api/v1/ekyc/initiate`, payload, params);
    ekycInitiateTrend.add(Date.now() - start);

    const success = check(res, {
      'ekyc initiate status is 200': (r) => r.status === 200,
      'ekyc response has verification_id': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.verification_id && body.verification_id.startsWith('IDV_');
        } catch {
          return false;
        }
      },
      'ekyc response has authorization_url': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.authorization_url && body.authorization_url.includes('authorize');
        } catch {
          return false;
        }
      },
    });
    errorRate.add(!success);
  });

  sleep(0.5);

  group('eKYC Status', () => {
    // Use a mock verification ID for status check
    const verificationId = 'IDV_01HXK123456789ABCDEF';
    const res = http.get(`${baseUrl}/api/v1/ekyc/status/${verificationId}`);

    const success = check(res, {
      'ekyc status returns 200': (r) => r.status === 200,
      'ekyc status has verification_id': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.verification_id === verificationId;
        } catch {
          return false;
        }
      },
    });
    errorRate.add(!success);
  });

  sleep(1);
}

/**
 * Teardown function - runs once after all VUs finish
 */
export function teardown(data) {
  console.log('Load test completed');
}
