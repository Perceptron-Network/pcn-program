import { PublicKey } from "@solana/web3.js";
import { PCN_PROGRAM_ID } from "./config";

function u64Le(value: bigint) {
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(value);
  return bytes;
}

function find(seeds: Array<Buffer | Uint8Array>) {
  return PublicKey.findProgramAddressSync(seeds, PCN_PROGRAM_ID)[0];
}

export const findConfigPda = () => find([Buffer.from("config")]);
export const findMintAuthorityPda = () => find([Buffer.from("mint_authority")]);
export const findSolReservePda = () => find([Buffer.from("sol_reserve")]);
export const findTokenReservePda = () => find([Buffer.from("token_reserve")]);
export const findEpochPda = (epochId: bigint) =>
  find([Buffer.from("epoch"), u64Le(epochId)]);
export const findEpochVaultPda = (epochId: bigint) =>
  find([Buffer.from("epoch_vault"), u64Le(epochId)]);
export const findClaimPda = (epochId: bigint, user: PublicKey) =>
  find([Buffer.from("claim"), u64Le(epochId), user.toBuffer()]);
