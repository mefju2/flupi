/**
 * Sandboxed JS evaluator using a hidden `<iframe sandbox="allow-scripts">`.
 * The iframe has a null origin and no access to the Tauri bridge, DOM storage,
 * or parent window. Communication is via postMessage/message.
 */

export interface EnvContext {
  name: string;
  variables: Record<string, string>;
}

export interface SandboxResult {
  result: string;
  mutations: Record<string, string>;
}

const SANDBOX_HTML = `<!doctype html><html><body><script>
window.addEventListener('message', function(e) {
  var id = e.data.id;
  var body = e.data.body;
  var args = e.data.args;
  var env = e.data.env || null;
  var mutations = {};
  var flu = {
    env: {
      name: function() { return env ? env.name : ''; },
      getVar: function(key) {
        if (!env || !env.variables) return '';
        return Object.prototype.hasOwnProperty.call(env.variables, key) ? env.variables[key] : '';
      },
      setVar: function(key, value) { mutations[key] = String(value); }
    }
  };
  try {
    var result = (new Function('args', 'flu', body))(args, flu);
    if (result === null || result === undefined) {
      parent.postMessage({ id: id, error: 'Function returned ' + result + '. Functions must return a string value.' }, '*');
    } else {
      parent.postMessage({ id: id, result: String(result), mutations: mutations }, '*');
    }
  } catch(err) {
    parent.postMessage({ id: id, error: err instanceof Error ? err.message : String(err) }, '*');
  }
});
<\/script></body></html>`;

let sandbox: HTMLIFrameElement | null = null;
let listenerRegistered = false;
const pending = new Map<string, { resolve: (v: SandboxResult) => void; reject: (e: Error) => void; timer: ReturnType<typeof setTimeout> }>();

function handleMessage(event: MessageEvent) {
  if (!sandbox || event.source !== sandbox.contentWindow) return;
  const { id, result, mutations, error } = event.data as { id: string; result?: string; mutations?: Record<string, string>; error?: string };
  const entry = pending.get(id);
  if (!entry) return;
  pending.delete(id);
  clearTimeout(entry.timer);
  if (error !== undefined) {
    entry.reject(new Error(error));
  } else {
    entry.resolve({ result: result!, mutations: mutations ?? {} });
  }
}

let sandboxReady: Promise<HTMLIFrameElement> | null = null;

function getSandbox(): Promise<HTMLIFrameElement> {
  if (sandboxReady) return sandboxReady;
  sandboxReady = new Promise((resolve) => {
    const iframe = document.createElement('iframe');
    iframe.setAttribute('sandbox', 'allow-scripts');
    iframe.style.display = 'none';
    iframe.srcdoc = SANDBOX_HTML;
    iframe.addEventListener('load', () => {
      sandbox = iframe;
      resolve(iframe);
    });
    document.body.appendChild(iframe);
    if (!listenerRegistered) {
      window.addEventListener('message', handleMessage);
      listenerRegistered = true;
    }
  });
  return sandboxReady;
}

/**
 * Evaluate `body` as a JS function body with `args` passed as the `args` array.
 * An optional `envContext` provides the active environment's data to `flu.env.*`.
 * Times out after 5 seconds; on timeout the iframe is destroyed so the next
 * call gets a clean sandbox.
 */
export async function evalInSandbox(body: string, args: string[], envContext?: EnvContext): Promise<SandboxResult> {
  const iframe = await getSandbox();
  return new Promise((resolve, reject) => {
    const id = Math.random().toString(36).slice(2) + Date.now().toString(36);
    const timer = setTimeout(() => {
      pending.delete(id);
      // Destroy stale iframe so next call starts fresh
      if (sandbox) {
        sandbox.remove();
        sandbox = null;
      }
      sandboxReady = null;
      reject(new Error('Function timed out after 5 seconds'));
    }, 5000);
    pending.set(id, { resolve, reject, timer });
    iframe.contentWindow!.postMessage({ id, body, args, env: envContext ?? null }, '*');
  });
}
