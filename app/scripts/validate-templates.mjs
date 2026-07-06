import { readdir, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import process from 'node:process';
import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..', '..');
const schemaPath = path.join(repoRoot, 'schemas', 'group-template.schema.json');
const templatesDir = path.join(repoRoot, 'fixtures', 'templates');

const ajv = new Ajv2020({ allErrors: true, strict: false });
addFormats(ajv);

const schema = JSON.parse(await readFile(schemaPath, 'utf8'));
const validate = ajv.compile(schema);
const templateFiles = (await readdir(templatesDir))
  .filter((fileName) => fileName.endsWith('.template.json'))
  .sort();

let failed = false;

for (const fileName of templateFiles) {
  const filePath = path.join(templatesDir, fileName);
  const data = JSON.parse(await readFile(filePath, 'utf8'));
  const valid = validate(data);
  if (valid) {
    console.log(`PASS ${path.relative(repoRoot, filePath)}`);
    continue;
  }

  failed = true;
  console.log(`FAIL ${path.relative(repoRoot, filePath)}`);
  for (const error of validate.errors ?? []) {
    const location = error.instancePath || '/';
    console.log(`  ${location} ${error.message}`);
  }
}

process.exitCode = failed ? 1 : 0;
