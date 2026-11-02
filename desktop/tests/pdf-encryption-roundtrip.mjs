import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { PDFDocument } from 'pdf-lib';
import * as pdfjsLib from 'pdfjs-dist/legacy/build/pdf.mjs';

const password = 'variscribe-memory-test-password';
const expected = { schemaVersion: 1, marker: 'attachment-roundtrip' };
const tauriDirectory = fileURLToPath(new URL('../src-tauri', import.meta.url));
const cargo = process.env.CARGO || join(process.env.USERPROFILE || '', '.cargo', 'bin', 'cargo.exe');

const plainDocument = await PDFDocument.create();
const testPage = plainDocument.addPage();
testPage.drawText('Synthetic Formora viewer compatibility test', { x: 48, y: 740, size: 14 });
await plainDocument.attach(
  new TextEncoder().encode(JSON.stringify(expected)),
  'formora-data.json',
  { mimeType: 'application/json' },
);
const plainBytes = await plainDocument.save();
const plainLoadingTask = pdfjsLib.getDocument({ data: plainBytes.slice() });
const plainLoadedDocument = await plainLoadingTask.promise;
const plainAttachments = await plainLoadedDocument.getAttachments();
const plainAttachmentEntry = plainAttachments && [...plainAttachments.entries()].find(([, item]) => item.filename === 'formora-data.json');

assert.ok(plainAttachmentEntry, 'an unprotected PDF should contain the editable JSON attachment');
const plainAttachmentContent = plainAttachmentEntry[1].content ?? await plainLoadedDocument.getAttachmentContent(plainAttachmentEntry[0]);
assert.deepEqual(JSON.parse(new TextDecoder().decode(plainAttachmentContent)), expected);
await plainLoadingTask.destroy();

const encryptedBytes = await encryptInMemory(plainBytes);
if (process.env.FORMORA_VIEWER_FIXTURE_PATH) {
  await writeFile(process.env.FORMORA_VIEWER_FIXTURE_PATH, encryptedBytes);
}
const encryptedLoadingTask = pdfjsLib.getDocument({
  data: encryptedBytes,
  password,
});
const loadedDocument = await encryptedLoadingTask.promise;
const attachments = await loadedDocument.getAttachments();
const attachmentEntry = attachments && [...attachments.entries()].find(([, item]) => item.filename === 'formora-data.json');

assert.ok(attachmentEntry, 'embedded JSON attachment should be present');
const attachmentContent = attachmentEntry[1].content ?? await loadedDocument.getAttachmentContent(attachmentEntry[0]);
assert.deepEqual(JSON.parse(new TextDecoder().decode(attachmentContent)), expected);
await encryptedLoadingTask.destroy();
const incorrectPasswordTask = pdfjsLib.getDocument({
  data: encryptedBytes,
  password: 'incorrect-password',
});
await assert.rejects(
  incorrectPasswordTask.promise,
  'an incorrect password should be rejected',
);
await incorrectPasswordTask.destroy();

console.log('PASS: unprotected and AES-256 encrypted attachment round-trips, plus wrong-password rejection');

function encryptInMemory(plainBytes) {
  return new Promise((resolve, reject) => {
    const process = spawn(cargo, ['run', '--quiet', '--example', 'qpdf-memory-stdio'], {
      cwd: tauriDirectory,
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    const encryptedChunks = [];
    const errorChunks = [];
    process.stdout.on('data', chunk => encryptedChunks.push(chunk));
    process.stderr.on('data', chunk => errorChunks.push(chunk));
    process.on('error', reject);
    process.on('close', exitCode => {
      if (exitCode !== 0) {
        reject(new Error(Buffer.concat(errorChunks).toString() || `QPDF helper exited with code ${exitCode}.`));
        return;
      }
      resolve(new Uint8Array(Buffer.concat(encryptedChunks)));
    });
    process.stdin.end(plainBytes);
  });
}
