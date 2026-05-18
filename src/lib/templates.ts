import * as client from "./api-client";

const LEGACY_STORAGE_KEY = "pebble-templates";

export interface EmailTemplate {
  id: string;
  name: string;
  subject: string;
  body: string;
  createdAt: number;
}

function clearLegacyTemplates() {
  try {
    localStorage.removeItem(LEGACY_STORAGE_KEY);
  } catch { /* ignored */ }
}

export async function listTemplates(): Promise<EmailTemplate[]> {
  const templates = await client.listEmailTemplates() as EmailTemplate[];
  clearLegacyTemplates();
  return templates;
}

export async function saveTemplate(template: Omit<EmailTemplate, "id" | "createdAt">): Promise<EmailTemplate> {
  const saved = await client.saveEmailTemplate(template) as EmailTemplate;
  clearLegacyTemplates();
  return saved;
}

export async function deleteTemplate(id: string): Promise<void> {
  await client.deleteEmailTemplate(id);
  clearLegacyTemplates();
}
