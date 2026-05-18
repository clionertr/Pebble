import * as client from "./api-client";

const LEGACY_STORAGE_KEY = "pebble-signatures";

function clearLegacySignatures() {
  try {
    localStorage.removeItem(LEGACY_STORAGE_KEY);
  } catch { /* ignored */ }
}

export async function getSignature(accountId: string): Promise<string> {
  const signature = await client.getEmailSignature(accountId);
  clearLegacySignatures();
  return signature;
}

export async function setSignature(accountId: string, signature: string): Promise<void> {
  await client.setEmailSignature(accountId, signature);
  clearLegacySignatures();
}
