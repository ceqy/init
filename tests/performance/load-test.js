// K6 性能测试脚本 - API 负载测试
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// 自定义指标
const errorRate = new Rate('errors');
const loginDuration = new Trend('login_duration');
const apiDuration = new Trend('api_duration');
const successfulLogins = new Counter('successful_logins');

// 测试配置
export const options = {
  // 测试场景
  scenarios: {
    // 场景 1: 渐进式负载测试
    ramp_up: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 50 },   // 2 分钟内增加到 50 用户
        { duration: '5m', target: 50 },   // 保持 50 用户 5 分钟
        { duration: '2m', target: 100 },  // 2 分钟内增加到 100 用户
        { duration: '5m', target: 100 },  // 保持 100 用户 5 分钟
        { duration: '2m', target: 200 },  // 2 分钟内增加到 200 用户
        { duration: '5m', target: 200 },  // 保持 200 用户 5 分钟
        { duration: '2m', target: 0 },    // 2 分钟内降到 0
      ],
      gracefulRampDown: '30s',
    },

    // 场景 2: 峰值测试
    spike: {
      executor: 'ramping-vus',
      startTime: '25m',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 500 },  // 10 秒内激增到 500 用户
        { duration: '1m', target: 500 },   // 保持 1 分钟
        { duration: '10s', target: 0 },    // 10 秒内降到 0
      ],
    },

    // 场景 3: 浸泡测试（长时间稳定负载）
    soak: {
      executor: 'constant-vus',
      startTime: '30m',
      vus: 100,
      duration: '30m',
    },
  },

  // 性能阈值
  thresholds: {
    // HTTP 请求失败率 < 1%
    'http_req_failed': ['rate<0.01'],

    // 95% 的请求在 500ms 内完成
    'http_req_duration': ['p(95)<500'],

    // 99% 的请求在 1000ms 内完成
    'http_req_duration': ['p(99)<1000'],

    // 错误率 < 1%
    'errors': ['rate<0.01'],

    // 登录接口 95% 在 300ms 内完成
    'login_duration': ['p(95)<300'],

    // API 接口 95% 在 200ms 内完成
    'api_duration': ['p(95)<200'],
  },
};

// 测试配置
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const TEST_USERS = 100; // 测试用户数量

// 生成测试用户
function getTestUser() {
  const userId = Math.floor(Math.random() * TEST_USERS);
  return {
    email: `test${userId}@example.com`,
    password: 'Test123456!',
  };
}

// 主测试函数
export default function () {
  const user = getTestUser();

  // 1. 登录测试
  const loginStart = Date.now();
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify(user), {
    headers: { 'Content-Type': 'application/json' },
    tags: { name: 'Login' },
  });

  const loginSuccess = check(loginRes, {
    'login status is 200': (r) => r.status === 200,
    'login has token': (r) => r.json('access_token') !== undefined,
    'login response time < 500ms': (r) => r.timings.duration < 500,
  });

  if (!loginSuccess) {
    errorRate.add(1);
    return;
  }

  loginDuration.add(Date.now() - loginStart);
  successfulLogins.add(1);

  const token = loginRes.json('access_token');

  // 2. 获取用户信息
  const apiStart = Date.now();
  const userRes = http.get(`${BASE_URL}/api/users/me`, {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
    tags: { name: 'GetUserInfo' },
  });

  check(userRes, {
    'user info status is 200': (r) => r.status === 200,
    'user info has email': (r) => r.json('email') !== undefined,
    'user info response time < 300ms': (r) => r.timings.duration < 300,
  }) || errorRate.add(1);

  apiDuration.add(Date.now() - apiStart);

  // 3. 列表查询测试
  const listRes = http.get(`${BASE_URL}/api/users?page=1&limit=20`, {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
    tags: { name: 'ListUsers' },
  });

  check(listRes, {
    'list status is 200': (r) => r.status === 200,
    'list has data': (r) => r.json('data') !== undefined,
    'list response time < 400ms': (r) => r.timings.duration < 400,
  }) || errorRate.add(1);

  // 4. 创建资源测试（模拟写操作）
  const createRes = http.post(
    `${BASE_URL}/api/resources`,
    JSON.stringify({
      name: `Resource-${Date.now()}`,
      description: 'Test resource',
    }),
    {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      tags: { name: 'CreateResource' },
    }
  );

  check(createRes, {
    'create status is 201': (r) => r.status === 201,
    'create has id': (r) => r.json('id') !== undefined,
  }) || errorRate.add(1);

  // 5. 更新资源测试
  if (createRes.status === 201) {
    const resourceId = createRes.json('id');
    const updateRes = http.put(
      `${BASE_URL}/api/resources/${resourceId}`,
      JSON.stringify({
        name: `Updated-${Date.now()}`,
      }),
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        tags: { name: 'UpdateResource' },
      }
    );

    check(updateRes, {
      'update status is 200': (r) => r.status === 200,
    }) || errorRate.add(1);

    // 6. 删除资源测试
    const deleteRes = http.del(`${BASE_URL}/api/resources/${resourceId}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
      },
      tags: { name: 'DeleteResource' },
    });

    check(deleteRes, {
      'delete status is 204': (r) => r.status === 204,
    }) || errorRate.add(1);
  }

  // 模拟用户思考时间
  sleep(Math.random() * 2 + 1); // 1-3 秒随机延迟
}

// 测试设置阶段
export function setup() {
  console.log('🚀 开始性能测试');
  console.log(`目标地址: ${BASE_URL}`);
  console.log(`测试用户数: ${TEST_USERS}`);

  // 健康检查
  const healthRes = http.get(`${BASE_URL}/health`);
  if (healthRes.status !== 200) {
    throw new Error('服务不可用，测试终止');
  }

  return { startTime: Date.now() };
}

// 测试清理阶段
export function teardown(data) {
  const duration = (Date.now() - data.startTime) / 1000;
  console.log(`✅ 测试完成，总耗时: ${duration.toFixed(2)} 秒`);
}

// 自定义摘要报告
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'performance-report.json': JSON.stringify(data),
    'performance-report.html': htmlReport(data),
  };
}

// 文本摘要
function textSummary(data, options) {
  const indent = options.indent || '';
  const enableColors = options.enableColors || false;

  let summary = '\n';
  summary += `${indent}性能测试摘要\n`;
  summary += `${indent}${'='.repeat(50)}\n\n`;

  // 请求统计
  const httpReqs = data.metrics.http_reqs;
  summary += `${indent}总请求数: ${httpReqs.values.count}\n`;
  summary += `${indent}请求速率: ${httpReqs.values.rate.toFixed(2)} req/s\n\n`;

  // 响应时间
  const httpDuration = data.metrics.http_req_duration;
  summary += `${indent}响应时间:\n`;
  summary += `${indent}  平均: ${httpDuration.values.avg.toFixed(2)} ms\n`;
  summary += `${indent}  最小: ${httpDuration.values.min.toFixed(2)} ms\n`;
  summary += `${indent}  最大: ${httpDuration.values.max.toFixed(2)} ms\n`;
  summary += `${indent}  P95: ${httpDuration.values['p(95)'].toFixed(2)} ms\n`;
  summary += `${indent}  P99: ${httpDuration.values['p(99)'].toFixed(2)} ms\n\n`;

  // 错误率
  const httpFailed = data.metrics.http_req_failed;
  const errorRate = (httpFailed.values.rate * 100).toFixed(2);
  summary += `${indent}错误率: ${errorRate}%\n\n`;

  // 自定义指标
  if (data.metrics.successful_logins) {
    summary += `${indent}成功登录: ${data.metrics.successful_logins.values.count}\n`;
  }

  return summary;
}

// HTML 报告
function htmlReport(data) {
  return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>性能测试报告</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 20px; }
    h1 { color: #333; }
    table { border-collapse: collapse; width: 100%; margin: 20px 0; }
    th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
    th { background-color: #4CAF50; color: white; }
    tr:nth-child(even) { background-color: #f2f2f2; }
    .pass { color: green; font-weight: bold; }
    .fail { color: red; font-weight: bold; }
  </style>
</head>
<body>
  <h1>Cuba ERP 性能测试报告</h1>
  <p>测试时间: ${new Date().toLocaleString()}</p>

  <h2>测试摘要</h2>
  <table>
    <tr><th>指标</th><th>值</th></tr>
    <tr><td>总请求数</td><td>${data.metrics.http_reqs.values.count}</td></tr>
    <tr><td>请求速率</td><td>${data.metrics.http_reqs.values.rate.toFixed(2)} req/s</td></tr>
    <tr><td>平均响应时间</td><td>${data.metrics.http_req_duration.values.avg.toFixed(2)} ms</td></tr>
    <tr><td>P95 响应时间</td><td>${data.metrics.http_req_duration.values['p(95)'].toFixed(2)} ms</td></tr>
    <tr><td>P99 响应时间</td><td>${data.metrics.http_req_duration.values['p(99)'].toFixed(2)} ms</td></tr>
    <tr><td>错误率</td><td>${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%</td></tr>
  </table>

  <h2>阈值检查</h2>
  <table>
    <tr><th>阈值</th><th>状态</th></tr>
    ${Object.entries(data.thresholds || {}).map(([name, result]) => `
      <tr>
        <td>${name}</td>
        <td class="${result.ok ? 'pass' : 'fail'}">${result.ok ? '✓ 通过' : '✗ 失败'}</td>
      </tr>
    `).join('')}
  </table>
</body>
</html>
  `;
}
