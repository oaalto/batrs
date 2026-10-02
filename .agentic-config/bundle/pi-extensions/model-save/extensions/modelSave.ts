import { mkdir, readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import type { ExtensionAPI, ExtensionCommandContext } from "@earendil-works/pi-coding-agent";
import type { Model } from "@earendil-works/pi-ai";
import type { AutocompleteItem } from "@earendil-works/pi-tui";

type PiSettings = Record<string, unknown>;

type StartupDefaults = {
  defaultProvider: string;
  defaultModel: string;
};

type SettingsTarget = {
  commandName: string;
  description: string;
  path: () => string;
  pickerTitle: string;
  successScope: string;
};

const GLOBAL_SETTINGS_TARGET: SettingsTarget = {
  commandName: "model-save",
  description: "Choose the current model and save it as the global startup default",
  path: () => join(homedir(), ".pi", "agent", "settings.json"),
  pickerTitle: "Save startup model",
  successScope: "global"
};

const LOCAL_SETTINGS_TARGET: SettingsTarget = {
  commandName: "model-save-local",
  description: "Choose the current model and save it as the project startup default",
  path: () => join(process.cwd(), ".pi", "settings.json"),
  pickerTitle: "Save project startup model",
  successScope: "project"
};

export const SETTINGS_PATH = GLOBAL_SETTINGS_TARGET.path();
export const LOCAL_SETTINGS_PATH = (): string => LOCAL_SETTINGS_TARGET.path();

export type ModelChoice = {
  provider: string;
  id: string;
  name?: string;
};

/** Return a human-friendly label for the selected provider/model pair. */
function describeModel(provider: string, modelId: string, name?: string): string {
  return name && name !== modelId ? `${name} (${provider}/${modelId})` : `${provider}/${modelId}`;
}

function modelRef(model: Pick<ModelChoice, "provider" | "id">): string {
  return `${model.provider}/${model.id}`;
}

function toModelChoice(model: Pick<Model<unknown>, "provider" | "id" | "name">): ModelChoice {
  return {
    provider: model.provider,
    id: model.id,
    name: model.name
  };
}

function getModels(ctx: ExtensionCommandContext): ModelChoice[] {
  return ctx.modelRegistry.getAll().map(toModelChoice);
}

function getModelSearchText(model: ModelChoice): string {
  return [modelRef(model), model.id, model.name].filter((value): value is string => Boolean(value)).join(" ");
}

function rankModels(models: ModelChoice[], query: string): ModelChoice[] {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return models;
  }

  const scored = models
    .map((model, index) => {
      const ref = modelRef(model).toLowerCase();
      const id = model.id.toLowerCase();
      const name = model.name?.toLowerCase() ?? "";
      const haystack = getModelSearchText(model).toLowerCase();

      let score = 0;
      if (ref === normalized) {
        score = 6;
      } else if (id === normalized || name === normalized) {
        score = 5;
      } else if (ref.startsWith(normalized)) {
        score = 4;
      } else if (id.startsWith(normalized) || name.startsWith(normalized)) {
        score = 3;
      } else if (ref.includes(normalized)) {
        score = 2;
      } else if (haystack.includes(normalized)) {
        score = 1;
      }

      return score > 0 ? { model, score, index } : null;
    })
    .filter((entry): entry is { model: ModelChoice; score: number; index: number } => entry !== null);

  return scored.sort((a, b) => b.score - a.score || a.index - b.index).map((entry) => entry.model);
}

export function createModelCompletions(models: ModelChoice[], prefix: string): AutocompleteItem[] | null {
  const ranked = rankModels(models, prefix).slice(0, 10);
  if (ranked.length === 0) {
    return null;
  }

  return ranked.map((model) => ({
    value: modelRef(model),
    label: model.id,
    description: model.provider
  }));
}

export function resolveModel(models: ModelChoice[], input: string): ModelChoice | null {
  const normalized = input.trim().toLowerCase();
  if (!normalized) {
    return null;
  }

  const exact = models.find((model) => modelRef(model).toLowerCase() === normalized);
  if (exact) {
    return exact;
  }

  const ranked = rankModels(models, normalized);
  return ranked.length === 1 ? ranked[0] : null;
}

/**
 * Load Pi settings without altering unrelated keys.
 *
 * Missing settings files are treated as empty settings. Invalid or non-object
 * JSON is a hard failure so the save commands never overwrite a broken file.
 */
async function readSettings(path: string): Promise<PiSettings> {
  try {
    const raw = await readFile(path, "utf8");
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      throw new Error(`Settings file must contain a JSON object: ${path}`);
    }
    return parsed as PiSettings;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return {};
    }
    throw error;
  }
}

async function writeSettings(path: string, settings: PiSettings): Promise<void> {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, `${JSON.stringify(settings, null, 2)}\n`, "utf8");
}

/** Persist only the startup model defaults for the target settings file, preserving all other settings. */
export async function persistStartupDefaults(defaults: StartupDefaults, path = SETTINGS_PATH): Promise<void> {
  const settings = await readSettings(path);
  const next: PiSettings = {
    ...settings,
    defaultProvider: defaults.defaultProvider,
    defaultModel: defaults.defaultModel
  };
  await writeSettings(path, next);
}

async function runModelSelector(
  ctx: ExtensionCommandContext,
  title: string,
  models: ModelChoice[]
): Promise<ModelChoice | null> {
  const options = models.slice(0, 10).map((model) => describeModel(model.provider, model.id, model.name));
  const selected = await ctx.ui.select(title, options, {
    initialValue: ctx.model ? describeModel(ctx.model.provider, ctx.model.id, ctx.model.name) : undefined,
    fuzzy: true,
    pageSize: 12,
    canCancel: true
  });
  if (!selected) {
    return null;
  }

  return models.find((model) => describeModel(model.provider, model.id, model.name) === selected) ?? null;
}

async function handleSaveCommand(
  pi: ExtensionAPI,
  ctx: ExtensionCommandContext,
  target: SettingsTarget,
  args: string
): Promise<void> {
  if (ctx.mode !== "tui") {
    ctx.ui.notify(`/${target.commandName} requires interactive mode`, "error");
    return;
  }

  const models = getModels(ctx);
  const picked = resolveModel(models, args) ?? (await runModelSelector(ctx, target.pickerTitle, models));
  if (!picked) {
    return;
  }

  const model = ctx.modelRegistry.find(picked.provider, picked.id);
  if (!model) {
    ctx.ui.notify(`Model not found: ${picked.provider}/${picked.id}`, "error");
    return;
  }

  const switched = await pi.setModel(model);
  if (!switched) {
    ctx.ui.notify(`No API key for ${picked.provider}/${picked.id}`, "error");
    return;
  }

  try {
    await persistStartupDefaults({ defaultProvider: picked.provider, defaultModel: picked.id }, target.path());
  } catch (error) {
    ctx.ui.notify(
      `Switched to ${picked.provider}/${picked.id}, but failed to save ${target.successScope} startup default: ${error instanceof Error ? error.message : String(error)}`,
      "error"
    );
    return;
  }

  ctx.ui.notify(
    `Saved ${target.successScope} startup model: ${describeModel(picked.provider, picked.id, picked.name)}`,
    "info"
  );
}

export default function modelSave(pi: ExtensionAPI): void {
  let latestModels: ModelChoice[] = [];

  pi.on("session_start", (_event, ctx) => {
    latestModels = getModels(ctx);
  });

  pi.on("model_select", (_event, ctx) => {
    latestModels = getModels(ctx);
  });

  for (const target of [GLOBAL_SETTINGS_TARGET, LOCAL_SETTINGS_TARGET]) {
    pi.registerCommand(target.commandName, {
      description: target.description,
      getArgumentCompletions: (prefix) => createModelCompletions(latestModels, prefix),
      handler: async (args, ctx) => {
        latestModels = getModels(ctx);
        await handleSaveCommand(pi, ctx, target, args);
      }
    });
  }
}
