// Copyright (c) 2023 Cloudflare, Inc.
// Licensed under the Apache 2.0 license found in the LICENSE file or at:
//     https://opensource.org/licenses/Apache-2.0

import assert from 'node:assert';
import util from 'node:util';

let scheduledLastCtrl;

export default {
  async fetch(request, env, ctx) {
    const { pathname } = new URL(request.url);
    if (pathname === '/body-length') {
      return Response.json(Object.fromEntries(request.headers));
    }
    if (pathname === '/web-socket') {
      const pair = new WebSocketPair();
      pair[0].addEventListener('message', (event) => {
        pair[0].send(util.inspect(event));
      });
      pair[0].accept();
      return new Response(null, {
        status: 101,
        webSocket: pair[1],
      });
    }
    return new Response(null, { status: 404 });
  },

  async scheduled(ctrl, env, ctx) {
    scheduledLastCtrl = ctrl;
    if (ctrl.cron === '* * * * 30') ctrl.noRetry();
  },

  async test(ctrl, env, ctx) {
    // Call `fetch()` with known body length
    {
      const body = new FixedLengthStream(3);
      const writer = body.writable.getWriter();
      void writer.write(new Uint8Array([1, 2, 3]));
      void writer.close();
      const response = await env.SERVICE.fetch(
        'http://placeholder/body-length',
        {
          method: 'POST',
          body: body.readable,
        }
      );
      const headers = new Headers(await response.json());
      assert.strictEqual(headers.get('Content-Length'), '3');
      assert.strictEqual(headers.get('Transfer-Encoding'), null);
    }

    // Check `fetch()` with unknown body length
    {
      const body = new IdentityTransformStream();
      const writer = body.writable.getWriter();
      void writer.write(new Uint8Array([1, 2, 3]));
      void writer.close();
      const response = await env.SERVICE.fetch(
        'http://placeholder/body-length',
        {
          method: 'POST',
          body: body.readable,
        }
      );
      const headers = new Headers(await response.json());
      assert.strictEqual(headers.get('Content-Length'), null);
      assert.strictEqual(headers.get('Transfer-Encoding'), 'chunked');
    }

    // Call `scheduled()` with no options
    {
      const result = await env.SERVICE.scheduled();
      assert.strictEqual(result.outcome, 'ok');
      assert(!result.noRetry);
      assert(Math.abs(Date.now() - scheduledLastCtrl.scheduledTime) < 3_000);
      assert.strictEqual(scheduledLastCtrl.cron, '');
    }

    // Call `scheduled()` with options, and noRetry()
    {
      const result = await env.SERVICE.scheduled({
        scheduledTime: 1000,
        cron: '* * * * 30',
      });
      assert.strictEqual(result.outcome, 'ok');
      assert(result.noRetry);
      assert.strictEqual(scheduledLastCtrl.scheduledTime, 1000);
      assert.strictEqual(scheduledLastCtrl.cron, '* * * * 30');
    }
  },
};

// inspect tests
export const test = {
  async test(ctrl, env, ctx) {
    // Check URL with duplicate search param keys
    const url = new URL('http://user:pass@placeholder:8787/path?a=1&a=2&b=3');
    assert.strictEqual(
      util.inspect(url),
      `URL {
  origin: 'http://placeholder:8787',
  href: 'http://user:pass@placeholder:8787/path?a=1&a=2&b=3',
  protocol: 'http:',
  username: 'user',
  password: 'pass',
  host: 'placeholder:8787',
  hostname: 'placeholder',
  port: '8787',
  pathname: '/path',
  search: '?a=1&a=2&b=3',
  hash: '',
  searchParams: URLSearchParams(3) { 'a' => '1', 'a' => '2', 'b' => '3' }
}`
    );

    // Check FormData with lower depth
    const formData = new FormData();
    formData.set('string', 'hello');
    formData.set(
      'blob',
      new Blob(['<h1>BLOB</h1>'], {
        type: 'text/html',
      })
    );
    formData.set(
      'file',
      new File(['password123'], 'passwords.txt', {
        type: 'text/plain',
        lastModified: 1000,
      })
    );
    assert.strictEqual(
      util.inspect(formData, { depth: 0 }),
      `FormData(3) { 'string' => 'hello', 'blob' => [File], 'file' => [File] }`
    );

    // Check request with mutable headers
    const request = new Request('http://placeholder', {
      method: 'POST',
      body: 'message',
      headers: { 'Content-Type': 'text/plain' },
    });
    // cache and CORS properties are now always present for structural type compatibility
    assert.strictEqual(
      util.inspect(request),
      `Request {
  method: 'POST',
  url: 'http://placeholder',
  headers: Headers(1) { 'content-type' => 'text/plain', [immutable]: false },
  redirect: 'follow',
  fetcher: null,
  signal: AbortSignal { aborted: false, reason: undefined, onabort: null },
  cf: undefined,
  integrity: '',
  keepalive: false,
  mode: 'no-cors',
  credentials: 'same-origin',
  destination: '',
  referrer: 'about:client',
  referrerPolicy: '',
  cache: 'default',
  body: ReadableStream {
    locked: false,
    [state]: 'readable',
    [supportsBYOB]: true,
    [length]: 7n
  },
  bodyUsed: false
}`
    );

    // Check response with immutable headers
    const response = await env.SERVICE.fetch('http://placeholder/not-found');
    assert.strictEqual(
      util.inspect(response),
      `Response {
  status: 404,
  statusText: 'Not Found',
  headers: Headers(0) { [immutable]: true },
  ok: false,
  redirected: false,
  url: 'http://placeholder/not-found',
  webSocket: null,
  cf: undefined,
  type: 'default',
  body: ReadableStream {
    locked: false,
    [state]: 'readable',
    [supportsBYOB]: true,
    [length]: 0n
  },
  bodyUsed: false
}`
    );

    // Check `MessageEvent` with unimplemented properties
    const webSocketResponse = await env.SERVICE.fetch(
      'http://placeholder/web-socket',
      {
        headers: { Upgrade: 'websocket' },
      }
    );
    const webSocket = webSocketResponse.webSocket;
    assert.notStrictEqual(webSocket, null);
    const messagePromise = new Promise((resolve) => {
      webSocket.addEventListener('message', (event) => {
        assert.strictEqual(
          event.data,
          `MessageEvent {
  ports: [ [length]: 0 ],
  source: null,
  lastEventId: '',
  origin: null,
  data: 'data',
  type: 'message',
  eventPhase: 2,
  composed: false,
  bubbles: false,
  cancelable: false,
  defaultPrevented: false,
  returnValue: true,
  currentTarget: WebSocket { readyState: 1, url: null, protocol: '', extensions: '' },
  target: WebSocket { readyState: 1, url: null, protocol: '', extensions: '' },
  srcElement: WebSocket { readyState: 1, url: null, protocol: '', extensions: '' },
  timeStamp: 0,
  isTrusted: true,
  cancelBubble: false,
  NONE: 0,
  CAPTURING_PHASE: 1,
  AT_TARGET: 2,
  BUBBLING_PHASE: 3
}`
        );
        resolve();
      });
    });
    webSocket.accept();
    webSocket.send('data');
    webSocket.close();
    await messagePromise;

    // Test sending to oversized URL (bigger than MAX_TRACE_BYTES), relevant primarily for tail worker test.
    await env.SERVICE.fetch('http://placeholder/' + '0'.repeat(2 ** 18));
  },
};

async function assertRequestCacheThrowsError(
  cacheHeader,
  errorName = 'Error',
  errorMessage = "The 'cache' field on 'RequestInitializerDict' is not implemented."
) {
  assert.throws(
    () => {
      new Request('https://example.org', { cache: cacheHeader });
    },
    {
      name: errorName,
      message: errorMessage,
    }
  );
}

async function assertFetchCacheRejectsError(
  cacheHeader,
  errorName = 'Error',
  errorMessage = "The 'cache' field on 'RequestInitializerDict' is not implemented."
) {
  await assert.rejects(
    (async () => {
      await fetch('https://example.org', { cache: cacheHeader });
    })(),
    {
      name: errorName,
      message: errorMessage,
    }
  );
}

export const cacheMode = {
  async test(ctrl, env, ctx) {
    var failureCases = [
      'default',
      'force-cache',
      'no-cache',
      'only-if-cached',
      'reload',
      'unsupported',
    ];
    // cache property is now always present for structural type compatibility
    assert.strictEqual('cache' in Request.prototype, true);
    {
      // cache now returns "default" instead of undefined when not explicitly set
      const req = new Request('https://example.org', {});
      assert.strictEqual(req.cache, 'default');
    }
    if (!env.CACHE_ENABLED) {
      failureCases.push('no-store');
      for (const cacheMode in failureCases) {
        await assertRequestCacheThrowsError(cacheMode);
        await assertFetchCacheRejectsError(cacheMode);
      }
    } else {
      {
        const req = new Request('https://example.org', { cache: 'no-store' });
        assert.strictEqual(req.cache, 'no-store');
      }
      {
        const response = await env.SERVICE.fetch(
          'http://placeholder/not-found',
          { cache: 'no-store' }
        );
        assert.strictEqual(
          util.inspect(response),
          `Response {
  status: 404,
  statusText: 'Not Found',
  headers: Headers(0) { [immutable]: true },
  ok: false,
  redirected: false,
  url: 'http://placeholder/not-found',
  webSocket: null,
  cf: undefined,
  type: 'default',
  body: ReadableStream {
    locked: false,
    [state]: 'readable',
    [supportsBYOB]: true,
    [length]: 0n
  },
  bodyUsed: false
}`
        );
      }
      for (const cacheMode in failureCases) {
        await assertRequestCacheThrowsError(
          cacheMode,
          'TypeError',
          'Unsupported cache mode: ' + cacheMode
        );
        await assertFetchCacheRejectsError(
          cacheMode,
          'TypeError',
          'Unsupported cache mode: ' + cacheMode
        );
      }
    }
  },
};

// Test for CORS-related properties and structural type compatibility with lib.dom.d.ts
export const corsProperties = {
  async test(ctrl, env, ctx) {
    const req = new Request('https://example.org');

    // These properties are now always present for structural type compatibility
    // with lib.dom.d.ts Request interface (same approach as Deno and Bun)
    assert.strictEqual('mode' in req, true);
    assert.strictEqual('credentials' in req, true);
    assert.strictEqual('destination' in req, true);
    assert.strictEqual('referrer' in req, true);
    assert.strictEqual('referrerPolicy' in req, true);

    // Verify spec-compliant default values
    assert.strictEqual(req.mode, 'no-cors');
    assert.strictEqual(req.credentials, 'same-origin');
    assert.strictEqual(req.destination, '');
    assert.strictEqual(req.referrer, 'about:client');
    assert.strictEqual(req.referrerPolicy, '');

    // Values should be the same for cloned requests
    const cloned = req.clone();
    assert.strictEqual(cloned.mode, 'no-cors');
    assert.strictEqual(cloned.credentials, 'same-origin');
    assert.strictEqual(cloned.destination, '');
    assert.strictEqual(cloned.referrer, 'about:client');
    assert.strictEqual(cloned.referrerPolicy, '');

    // Values should be the same when reconstructing from another request
    const reconstructed = new Request(req);
    assert.strictEqual(reconstructed.mode, 'no-cors');
    assert.strictEqual(reconstructed.credentials, 'same-origin');
    assert.strictEqual(reconstructed.destination, '');
    assert.strictEqual(reconstructed.referrer, 'about:client');
    assert.strictEqual(reconstructed.referrerPolicy, '');

    // CORS-related properties in RequestInit should be silently ignored
    // (per WinterTC guidance for non-browser runtimes)
    const reqWithInit = new Request('https://example.org', {
      mode: 'cors',
      credentials: 'include',
      referrer: 'https://example.com',
      referrerPolicy: 'no-referrer',
    });
    // Values should still be the spec-compliant defaults, not the init values
    assert.strictEqual(reqWithInit.mode, 'no-cors');
    assert.strictEqual(reqWithInit.credentials, 'same-origin');
    assert.strictEqual(reqWithInit.destination, '');
    assert.strictEqual(reqWithInit.referrer, 'about:client');
    assert.strictEqual(reqWithInit.referrerPolicy, '');
  },
};
