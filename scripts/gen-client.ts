import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor as renderJavaScriptVisitor } from "@codama/renderers-js";
import { renderVisitor as renderRustVisitor } from "@codama/renderers-rust";
import { mkdirSync, readFileSync } from "fs";
import { basename, dirname } from "path";

const CANONICAL_PCN_PROGRAM_ID = "86oGodFG8DfLYHAgYwUPCNQzaods7Auxz9cnU7XzWipt";

const RUST_DEPENDENCY_MAP = {
  generated: "crate::codama_rust",
  generatedAccounts: "crate::codama_rust::accounts",
  generatedErrors: "crate::codama_rust::errors",
  generatedInstructions: "crate::codama_rust::instructions",
  generatedTypes: "crate::codama_rust::types",
} as const;

type AnchorIdl = {
  address?: string;
};

function expectedProgramId(): string {
  return (
    process.env.PCN_EXPECTED_PROGRAM_ID?.trim() || CANONICAL_PCN_PROGRAM_ID
  );
}

function validateProgramId(idl: AnchorIdl, idlPath: string): void {
  const expected = expectedProgramId();
  if (idl.address === expected) {
    return;
  }

  throw new Error(
    `Refusing to generate clients from ${idlPath}: IDL address ` +
      `${idl.address ?? "<missing>"} does not match ${expected}.`
  );
}

async function createClient(idlPath: string, rustPath: string, tsPath: string) {
  const anchorIdl = JSON.parse(readFileSync(idlPath, "utf8")) as AnchorIdl;
  validateProgramId(anchorIdl, idlPath);

  // Codama packages currently expose compatible runtime nodes through slightly
  // different TypeScript versions, so keep the mismatch at this boundary.
  // @ts-expect-error Codama root-node package versions do not share one type.
  const codama = createFromRoot(rootNodeFromAnchor(anchorIdl));

  mkdirSync(rustPath, { recursive: true });
  mkdirSync(tsPath, { recursive: true });

  console.log(`Generating Rust client at: ${rustPath}`);
  codama.accept(
    renderRustVisitor(dirname(rustPath), {
      dependencyMap: RUST_DEPENDENCY_MAP,
      generatedFolder: basename(rustPath),
      syncCargoToml: false,
    })
  );

  console.log(`Generating TypeScript client at: ${tsPath}`);
  await codama.accept(
    renderJavaScriptVisitor(dirname(tsPath), {
      generatedFolder: basename(tsPath),
      syncPackageJson: false,
    })
  );
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length !== 3 || args.includes("--help") || args.includes("-h")) {
    console.log(
      "Usage: gen-client <idl-path> <rust-output-path> <typescript-output-path>"
    );
    process.exit(args.length === 3 ? 0 : 1);
  }

  await createClient(args[0], args[1], args[2]);
  console.log("Done");
}

main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});
