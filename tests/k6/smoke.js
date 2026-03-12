/**
 * k6 Smoke Test for VaultPass Platform API
 *
 * Quick sanity check before running full load tests.
 * Runs a single VU for 10 seconds to verify endpoints work.
 *
 * Run with:
 *   k6 run tests/k6/smoke.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 1,
  duration: '10s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<1000'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
  // Health check
  let res = http.get(`${BASE_URL}/health`);
  check(res, {
    'health status is 200': (r) => r.status === 200,
  });

  sleep(0.5);

  // eKYC initiate
  res = http.post(
    `${BASE_URL}/api/v1/ekyc/initiate`,
    JSON.stringify({
      tenant_id: 'TNT_smoke_test',
    }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  check(res, {
    'ekyc initiate status is 200': (r) => r.status === 200,
  });

  sleep(0.5);

  // eKYC status
  res = http.get(`${BASE_URL}/api/v1/ekyc/status/IDV_smoke_test`);
  check(res, {
    'ekyc status is 200': (r) => r.status === 200,
  });

  sleep(0.5);

  // Bind DID (valid DID format)
  res = http.post(
    `${BASE_URL}/api/v1/ekyc/bind-did`,
    JSON.stringify({
      verification_id: 'IDV_smoke_test',
      did: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
    }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  check(res, {
    'bind-did status is 200': (r) => r.status === 200,
  });

  sleep(1);
}
