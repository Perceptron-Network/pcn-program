import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import ts from "typescript";
import { fileURLToPath } from "node:url";

const PROJECT_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../.."
);
const SRC_ROOT = path.join(PROJECT_ROOT, "src");
const WALLET_POPUP_CALLS = new Set([
  "connect",
  "sendTransaction",
  "signAllTransactions",
  "signMessage",
  "signTransaction",
]);

function collectRuntimeSourceFiles(directory: string): string[] {
  const files: string[] = [];
  for (const entry of readdirSync(directory)) {
    const absolutePath = path.join(directory, entry);
    if (statSync(absolutePath).isDirectory()) {
      files.push(...collectRuntimeSourceFiles(absolutePath));
    } else if (
      (entry.endsWith(".ts") || entry.endsWith(".tsx")) &&
      !entry.includes(".test.")
    ) {
      files.push(absolutePath);
    }
  }
  return files;
}

function callName(expression: ts.Expression) {
  if (ts.isIdentifier(expression)) return expression.text;
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text;
  return null;
}

function walletCallsInsideEffect(filePath: string) {
  const source = ts.createSourceFile(
    filePath,
    readFileSync(filePath, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    filePath.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS
  );
  const findings: string[] = [];

  function visit(node: ts.Node, inEffect: boolean) {
    if (ts.isCallExpression(node)) {
      const name = callName(node.expression);
      const nextInEffect = inEffect || name === "useEffect";
      if (inEffect && name && WALLET_POPUP_CALLS.has(name)) {
        findings.push(`${path.relative(PROJECT_ROOT, filePath)}: ${name}`);
      }
      ts.forEachChild(node, (child) => visit(child, nextInEffect));
      return;
    }
    ts.forEachChild(node, (child) => visit(child, inEffect));
  }

  visit(source, false);
  return findings;
}

test("wallet connect, sign, and send calls stay outside useEffect", () => {
  const findings = collectRuntimeSourceFiles(SRC_ROOT).flatMap(
    walletCallsInsideEffect
  );
  assert.deepEqual(findings, []);
});
