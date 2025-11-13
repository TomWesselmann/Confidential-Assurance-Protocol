/**
 * k6 Load Test for /verify Endpoint - Week 4
 *
 * Targets:
 * - 50 RPS (Requests Per Second)
 * - p95 latency < 500ms
 * - Error rate < 1%
 *
 * Usage:
 *   k6 run k6/verify.js
 *
 * With custom environment:
 *   BASE=http://localhost:8080 TOKEN=<jwt> k6 run k6/verify.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const verifyLatency = new Trend('verify_latency', true);

// Load test configuration
export const options = {
  // Target: 50 RPS for 3 minutes
  scenarios: {
    constant_load: {
      executor: 'constant-arrival-rate',
      rate: 50,              // 50 RPS
      timeUnit: '1s',
      duration: '3m',        // 3 minutes
      preAllocatedVUs: 10,   // Pre-allocate VUs
      maxVUs: 50,            // Max concurrent VUs
    },
  },

  // Thresholds (DoD criteria)
  thresholds: {
    'http_req_duration': ['p(95)<500'],  // p95 < 500ms
    'errors': ['rate<0.01'],              // Error rate < 1%
    'http_req_failed': ['rate<0.01'],     // HTTP failures < 1%
  },

  // Summary output
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
};

// Environment variables with defaults
const BASE_URL = __ENV.BASE || 'http://localhost:8080';
const TOKEN = __ENV.TOKEN || 'mock-token-for-testing';

// Test data: Valid verification context (Mode B - Embedded IR)
const testPayload = {
  ir: {
    ir_version: '1.0',
    policy_id: 'lksg.v1',
    policy_hash: 'sha3-256:b98c3db55f874476dc749ea32b70bdf5369a0d7bc5364f236e034f1ddcd94638',
    rules: [
      {
        rule_id: 'no_sanctions',
        operator: 'non_membership',
        args: {
          set_var: 'supplier_hashes',
          element: { var: 'sanctions_root' }
        }
      }
    ],
    ir_hash: 'sha3-256:df3a3eeb7c72f6204131397e4b0a4b16235f1e20cc66102153ad6d4ee78f892c'
  },
  context: {
    supplier_hashes: [
      '0x1234567890123456789012345678901234567890123456789012345678901234',
      '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd'
    ],
    ubo_hashes: [
      '0x1111111111111111111111111111111111111111111111111111111111111111'
    ],
    company_commitment_root: null,
    sanctions_root: '0x0000000000000000000000000000000000000000000000000000000000000000',
    jurisdiction_root: null
  },
  backend: 'mock',
  options: {
    adaptive: false,
    check_timestamp: false,
    check_registry: false
  }
};

// HTTP request parameters
const params = {
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${TOKEN}`,
  },
  timeout: '5s',
};

export default function () {
  // Make POST request to /verify
  const startTime = new Date().getTime();
  const response = http.post(
    `${BASE_URL}/verify`,
    JSON.stringify(testPayload),
    params
  );
  const duration = new Date().getTime() - startTime;

  // Record custom metrics
  verifyLatency.add(duration);

  // Verify response
  const success = check(response, {
    'status is 200': (r) => r.status === 200,
    'response has result': (r) => r.json('result') !== undefined,
    'response has manifest_hash': (r) => r.json('manifest_hash') !== undefined,
    'response has proof_hash': (r) => r.json('proof_hash') !== undefined,
    'result is OK or FAIL': (r) => {
      const result = r.json('result');
      return result === 'OK' || result === 'FAIL';
    },
  });

  // Track errors
  errorRate.add(!success);

  // Think time (optional - adjust to control RPS)
  // Note: constant-arrival-rate executor handles rate limiting
  sleep(0.01);
}

// Setup function (runs once at start)
export function setup() {
  console.log(`🚀 Starting load test against ${BASE_URL}`);
  console.log(`🎯 Target: 50 RPS, p95 < 500ms, error rate < 1%`);
  console.log(`⏱️  Duration: 3 minutes`);

  // Warmup: Check if server is responsive
  const warmupResponse = http.get(`${BASE_URL}/healthz`);
  if (warmupResponse.status !== 200) {
    throw new Error(`Server not ready: ${warmupResponse.status}`);
  }
  console.log(`✅ Server health check passed`);

  return { startTime: new Date().toISOString() };
}

// Teardown function (runs once at end)
export function teardown(data) {
  console.log(`\n📊 Load test completed`);
  console.log(`   Started: ${data.startTime}`);
  console.log(`   Ended: ${new Date().toISOString()}`);
}

/**
 * Custom summary handler (optional)
 *
 * To use, run with: k6 run --summary-export=reports/load_week4.json k6/verify.js
 */
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
  };
}

function textSummary(data, options) {
  const indent = options?.indent || '';
  const enableColors = options?.enableColors || false;

  let summary = `\n${indent}📈 Load Test Summary\n`;
  summary += `${indent}${'='.repeat(50)}\n`;

  // Requests
  const requests = data.metrics.http_reqs?.values?.count || 0;
  const duration = data.state?.testRunDurationMs || 0;
  const rps = (requests / (duration / 1000)).toFixed(2);
  summary += `${indent}Total Requests: ${requests}\n`;
  summary += `${indent}Actual RPS: ${rps}\n`;

  // Latency
  const p95 = data.metrics.http_req_duration?.values?.['p(95)'] || 0;
  const p99 = data.metrics.http_req_duration?.values?.['p(99)'] || 0;
  const avg = data.metrics.http_req_duration?.values?.avg || 0;
  summary += `${indent}p95 Latency: ${p95.toFixed(2)}ms ${p95 < 500 ? '✅' : '❌'}\n`;
  summary += `${indent}p99 Latency: ${p99.toFixed(2)}ms\n`;
  summary += `${indent}Avg Latency: ${avg.toFixed(2)}ms\n`;

  // Errors
  const errorRateValue = data.metrics.errors?.values?.rate || 0;
  const failedRequests = data.metrics.http_req_failed?.values?.rate || 0;
  summary += `${indent}Error Rate: ${(errorRateValue * 100).toFixed(2)}% ${errorRateValue < 0.01 ? '✅' : '❌'}\n`;
  summary += `${indent}Failed Requests: ${(failedRequests * 100).toFixed(2)}%\n`;

  summary += `${indent}${'='.repeat(50)}\n`;

  return summary;
}
