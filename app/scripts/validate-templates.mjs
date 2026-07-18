// Validates persisted-format fixtures against the versioned JSON Schemas in
// schemas/ (I7): group templates, op-log records, embedded stories, embedded
// authored templates, and (compile-only until P2.3 emits instances) the
// snapshot data envelope. Any schema compile failure or instance validation
// failure fails the run loudly (I3).
import { readdir, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import process from 'node:process';
import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..', '..');
const schemasDir = path.join(repoRoot, 'schemas');
const templatesDir = path.join(repoRoot, 'fixtures', 'templates');
const groupsDir = path.join(repoRoot, 'fixtures', 'groups');

const SCHEMA_IDS = {
  groupTemplate:
    'https://community-navigator.example.test/schemas/group-template/0.1.0',
  opLog: 'https://community-navigator.example.test/schemas/op-log/0.1.0',
  storyPath:
    'https://community-navigator.example.test/schemas/story-path/0.1.0',
  snapshotEnvelope:
    'https://community-navigator.example.test/schemas/snapshot-envelope/0.1.0',
};

const MAX_PRINTED_ERRORS = 10;

const ajv = new Ajv2020({ allErrors: true, strict: false });
addFormats(ajv);

let failed = false;

function fail(label, messages) {
  failed = true;
  console.log(`FAIL ${label}`);
  for (const message of messages.slice(0, MAX_PRINTED_ERRORS)) {
    console.log(`  ${message}`);
  }
  if (messages.length > MAX_PRINTED_ERRORS) {
    console.log(
      `  ... and ${messages.length - MAX_PRINTED_ERRORS} more errors`,
    );
  }
}

function ajvErrors(validate) {
  return (validate.errors ?? []).map(
    (error) => `${error.instancePath || '/'} ${error.message}`,
  );
}

// Register every schema so cross-file $refs resolve; a schema that fails to
// compile fails the whole run.
const schemaFiles = (await readdir(schemasDir))
  .filter((fileName) => fileName.endsWith('.schema.json'))
  .sort();
for (const fileName of schemaFiles) {
  const filePath = path.join(schemasDir, fileName);
  try {
    ajv.addSchema(JSON.parse(await readFile(filePath, 'utf8')));
  } catch (error) {
    fail(path.relative(repoRoot, filePath), [`schema failed to load: ${error.message}`]);
  }
}

function compileRef(ref, label) {
  try {
    return ajv.compile({ $ref: ref });
  } catch (error) {
    fail(label, [`schema failed to compile: ${error.message}`]);
    return null;
  }
}

const validateTemplate = compileRef(SCHEMA_IDS.groupTemplate, 'group-template');
const validateOperation = compileRef(
  `${SCHEMA_IDS.opLog}#/$defs/operation`,
  'op-log operation',
);
const validateStory = compileRef(SCHEMA_IDS.storyPath, 'story-path');
const validateSnapshotEnvelope = compileRef(
  SCHEMA_IDS.snapshotEnvelope,
  'snapshot-envelope',
);

// 1. Authored group-template fixture files.
if (validateTemplate) {
  const templateFiles = (await readdir(templatesDir))
    .filter((fileName) => fileName.endsWith('.template.json'))
    .sort();
  for (const fileName of templateFiles) {
    const filePath = path.join(templatesDir, fileName);
    const label = path.relative(repoRoot, filePath);
    let data;
    try {
      data = JSON.parse(await readFile(filePath, 'utf8'));
    } catch (error) {
      fail(label, [`invalid JSON: ${error.message}`]);
      continue;
    }
    if (validateTemplate(data)) {
      console.log(`PASS ${label}`);
    } else {
      fail(label, ajvErrors(validateTemplate));
    }
  }
}

// 2. Op-log fixtures: every JSONL line is one operation record; embedded
// stories validate against story-path; embedded GroupCreate template_json
// validates against the authored group-template schema.
if (validateOperation && validateStory && validateTemplate) {
  const opsFiles = (await readdir(groupsDir))
    .filter((fileName) => fileName.endsWith('.ops.jsonl'))
    .sort();
  for (const fileName of opsFiles) {
    const filePath = path.join(groupsDir, fileName);
    const label = path.relative(repoRoot, filePath);
    const lines = (await readFile(filePath, 'utf8')).split('\n');
    const errors = [];
    let ops = 0;
    let stories = 0;
    let embeddedTemplates = 0;
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index].trim();
      if (line === '') {
        continue;
      }
      const lineNo = index + 1;
      let op;
      try {
        op = JSON.parse(line);
      } catch (error) {
        errors.push(`line ${lineNo}: invalid JSON: ${error.message}`);
        continue;
      }
      ops += 1;
      if (!validateOperation(op)) {
        for (const message of ajvErrors(validateOperation)) {
          errors.push(`line ${lineNo}: ${message}`);
        }
        continue;
      }
      if (op.kind?.op === 'StoryCreate') {
        stories += 1;
        if (!validateStory(op.kind.story)) {
          for (const message of ajvErrors(validateStory)) {
            errors.push(`line ${lineNo} (story): ${message}`);
          }
        }
      }
      if (op.kind?.op === 'GroupCreate') {
        embeddedTemplates += 1;
        let embedded;
        try {
          embedded = JSON.parse(op.kind.template_json);
        } catch (error) {
          errors.push(
            `line ${lineNo} (template_json): invalid JSON: ${error.message}`,
          );
          continue;
        }
        if (!validateTemplate(embedded)) {
          for (const message of ajvErrors(validateTemplate)) {
            errors.push(`line ${lineNo} (template_json): ${message}`);
          }
        }
      }
    }
    if (errors.length > 0) {
      fail(label, errors);
    } else {
      console.log(
        `PASS ${label} (${ops} ops, ${stories} stories, ${embeddedTemplates} embedded templates)`,
      );
    }
  }
}

// 3. Snapshot envelope: instances are produced at snapshot build time and
// validated there by app/scripts/embed-snapshot-data.ts (P2.3); no committed
// instances exist, so compiling above proves the schema itself is well-formed.
if (validateSnapshotEnvelope) {
  console.log(
    'PASS schemas/snapshot-envelope.schema.json (compile only - instances validated at snapshot build by embed-snapshot-data.ts)',
  );
}

process.exitCode = failed ? 1 : 0;
