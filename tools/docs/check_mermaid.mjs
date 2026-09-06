// Documentation-only parser: no generated assets and no browser executable.
import fs from 'node:fs';
import { JSDOM } from 'jsdom';

const dom = new JSDOM('<!DOCTYPE html><body></body>');
globalThis.window = dom.window;
globalThis.document = dom.window.document;
const { default: mermaid } = await import('mermaid');
mermaid.initialize({ startOnLoad: false, securityLevel: 'strict' });
let failed = false;
for (const { file, line, code } of JSON.parse(fs.readFileSync(0, 'utf8'))) {
  try {
    await mermaid.parse(code);
  } catch (error) {
    console.error(`${file}:${line}: invalid Mermaid: ${error.message ?? error}`);
    failed = true;
  }
}
dom.window.close();
process.exitCode = failed ? 1 : 0;
