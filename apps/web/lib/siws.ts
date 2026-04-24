import { Keypair, StrKey } from "@stellar/stellar-sdk";

export interface SiwsMessageParams {
  address: string;
  domain: string;
  statement?: string;
  issuedAt?: string;
  expirationTime?: string;
  nonce: string;
  network: string;
  version?: string;
}

export interface SiwsMessage {
  message: string;
  params: Required<SiwsMessageParams>;
}

export interface SiwsVerifyParams {
  message: string;
  signature: string;
  address: string;
}

export interface SiwsVerifyResult {
  valid: boolean;
  error?: string;
}

export function generateNonce(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export function buildSiwsMessage(params: SiwsMessageParams): SiwsMessage {
  const issuedAt = params.issuedAt ?? new Date().toISOString();
  const expirationTime =
    params.expirationTime ??
    new Date(Date.now() + 5 * 60 * 1000).toISOString();
  const statement =
    params.statement ?? "Sign in to Lance with your Stellar wallet.";
  const version = params.version ?? "1";

  const filled: Required<SiwsMessageParams> = {
    ...params,
    issuedAt,
    expirationTime,
    statement,
    version,
  };

  const message = [
    `${params.domain} wants you to sign in with your Stellar account:`,
    params.address,
    "",
    statement,
    "",
    `URI: https://${params.domain}`,
    `Version: ${version}`,
    `Network: ${params.network}`,
    `Nonce: ${params.nonce}`,
    `Issued At: ${issuedAt}`,
    `Expiration Time: ${expirationTime}`,
  ].join("\n");

  return { message, params: filled };
}

export function verifySiwsSignature({
  message,
  signature,
  address,
}: SiwsVerifyParams): SiwsVerifyResult {
  try {
    if (!StrKey.isValidEd25519PublicKey(address)) {
      return { valid: false, error: "Invalid Stellar public key format." };
    }

    const keypair = Keypair.fromPublicKey(address);
    const messageBytes = new TextEncoder().encode(message);
    const signatureBytes = Buffer.from(signature, "base64");

    const valid = keypair.verify(messageBytes, signatureBytes);
    return { valid };
  } catch (err) {
    return {
      valid: false,
      error:
        err instanceof Error ? err.message : "Signature verification failed.",
    };
  }
}

export function isSiwsMessageExpired(message: string): boolean {
  const match = /Expiration Time: (.+)/.exec(message);
  if (!match?.[1]) return true;
  return new Date(match[1]) < new Date();
}

export function extractNonce(message: string): string | null {
  const match = /Nonce: ([a-f0-9]+)/.exec(message);
  return match?.[1] ?? null;
}