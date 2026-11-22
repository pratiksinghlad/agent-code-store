import {
  closeSync,
  existsSync,
  fsyncSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeSync,
} from "node:fs";
import { dirname, join } from "node:path";

const HELP = `usage: json [--path <file>] [--json <text>] [--out <file>]

Read JSON and print it with sorted keys.
Write it atomically when --out is set.
--json overrides --path. With neither, read stdin.
`;

const FLAGS = new Set(["--path", "--json", "--out"]);

interface Options {
  path?: string;
  raw?: string;
  out?: string;
}

function main(): number {
  const argv = process.argv.slice(2);
  if (argv.includes("--help") || argv.includes("-h")) {
    process.stdout.write(HELP);
    return 0;
  }
  try {
    return execute(argv);
  } catch (error) {
    const message = error instanceof Error ? error.message : "failed";
    process.stderr.write(`${message}\n`);
    return 1;
  }
}

function execute(argv: string[]): number {
  const options = parseArgs(argv);
  const body = canonical(parseJson(loadText(options), sourceLabel(options)));
  if (options.out === undefined) {
    process.stdout.write(body);
    return 0;
  }
  atomicWrite(options.out, body);
  return 0;
}

function parseArgs(argv: string[]): Options {
  const options: Options = {};
  let index = 0;
  while (index < argv.length) {
    const flag = argv[index] ?? "";
    if (!FLAGS.has(flag)) {
      throw new Error(`unknown argument: ${flag}`);
    }
    assignFlag(options, flag, flagValue(argv, index, flag));
    index += 2;
  }
  return options;
}

function flagValue(argv: string[], index: number, flag: string): string {
  const value = argv[index + 1];
  if (value === undefined || value.startsWith("--")) {
    throw new Error(`missing value for ${flag}`);
  }
  return value;
}

function assignFlag(options: Options, flag: string, value: string): void {
  if (flag === "--path") {
    options.path = value;
    return;
  }
  if (flag === "--json") {
    options.raw = value;
    return;
  }
  options.out = value;
}

function sourceLabel(options: Options): string {
  if (options.raw !== undefined) {
    return "--json";
  }
  if (options.path !== undefined) {
    return options.path;
  }
  return "stdin";
}

function loadText(options: Options): string {
  if (options.raw !== undefined) {
    return options.raw;
  }
  if (options.path !== undefined) {
    return readText(options.path);
  }
  if (process.stdin.isTTY === true) {
    throw new Error("missing json input");
  }
  const text = readFileSync(0, "utf8");
  if (text === "") {
    throw new Error("missing json input");
  }
  return text;
}

function readText(path: string): string {
  let bytes: Uint8Array;
  try {
    bytes = readFileSync(path);
  } catch (error) {
    const message = error instanceof Error ? error.message : "read failed";
    throw new Error(message);
  }
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new Error(`file is not utf-8: ${path}`);
  }
}

function parseJson(text: string, label: string): unknown {
  try {
    return JSON.parse(text) as unknown;
  } catch (error) {
    const detail = error instanceof Error ? error.message : "invalid json";
    throw new Error(`invalid json: ${label}: ${detail}`);
  }
}

function canonical(value: unknown): string {
  return `${JSON.stringify(sortValue(value), null, 2)}\n`;
}

function sortValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((item: unknown) => sortValue(item));
  }
  if (value !== null && typeof value === "object") {
    return sortObject(value);
  }
  return value;
}

function sortObject(value: object): Record<string, unknown> {
  const source = value as Record<string, unknown>;
  const sorted: Record<string, unknown> = {};
  const keys = Object.keys(source).sort(compareKeys);
  for (const key of keys) {
    sorted[key] = sortValue(source[key]);
  }
  return sorted;
}

function compareKeys(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

function atomicWrite(path: string, content: string): void {
  const directory = dirname(path);
  mkdirSync(directory, { recursive: true });
  const temporary = createTemp(directory);
  try {
    writeSynced(temporary, content);
    renameSync(temporary, path);
  } catch (error) {
    removeTemp(temporary);
    throw error;
  }
}

function createTemp(directory: string): string {
  let attempt = 0;
  while (attempt < 100) {
    attempt += 1;
    const opened = openTemp(directory, attempt);
    if (opened !== undefined) {
      return opened;
    }
  }
  throw new Error("could not create a temp file");
}

function openTemp(directory: string, attempt: number): string | undefined {
  const temporary = join(directory, `.json-${process.pid}-${attempt}.tmp`);
  try {
    closeSync(openSync(temporary, "wx"));
  } catch (error) {
    if (isAlreadyExists(error)) {
      return undefined;
    }
    throw error;
  }
  return temporary;
}

function writeSynced(temporary: string, content: string): void {
  const handle = openSync(temporary, "w");
  try {
    writeSync(handle, content);
    fsyncSync(handle);
  } finally {
    closeSync(handle);
  }
}

function removeTemp(temporary: string): void {
  if (existsSync(temporary)) {
    unlinkSync(temporary);
  }
}

function isAlreadyExists(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "EEXIST"
  );
}

process.exitCode = main();
