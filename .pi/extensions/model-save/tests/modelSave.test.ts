import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import type { ExtensionCommandContext } from "@earendil-works/pi-coding-agent";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, describe, test } from "node:test";
import { pathToFileURL } from "node:url";

const tempHomes: string[] = [];
const extensionPath = join(dirname(new URL(import.meta.url).pathname), "..", "extensions", "modelSave.ts");

async function loadModuleWithHome(home: string) {
  process.env.HOME = home;
  process.env.USERPROFILE = home;
  return import(`${pathToFileURL(extensionPath).href}?home=${encodeURIComponent(home)}&t=${Date.now()}`);
}

afterEach(async () => {
  await Promise.all(tempHomes.splice(0).map((dir) => rm(dir, { recursive: true, force: true })));
});

describe("persistStartupDefaults", () => {
  test("creates the global settings file when missing", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, SETTINGS_PATH } = await loadModuleWithHome(home);

    await persistStartupDefaults({ defaultProvider: "aura", defaultModel: "gpt-4o" });

    const saved = JSON.parse(await readFile(SETTINGS_PATH, "utf8"));
    assert.equal(saved.defaultProvider, "aura");
    assert.equal(saved.defaultModel, "gpt-4o");
  });

  test("creates the local settings file when missing", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, LOCAL_SETTINGS_PATH } = await loadModuleWithHome(home);

    const projectDir = await mkdtemp(join(tmpdir(), "model-save-project-"));
    tempHomes.push(projectDir);
    const previousCwd = process.cwd();
    process.chdir(projectDir);

    try {
      await persistStartupDefaults({ defaultProvider: "aura", defaultModel: "gpt-4o-mini" }, LOCAL_SETTINGS_PATH());

      const saved = JSON.parse(await readFile(LOCAL_SETTINGS_PATH(), "utf8"));
      assert.equal(saved.defaultProvider, "aura");
      assert.equal(saved.defaultModel, "gpt-4o-mini");
    } finally {
      process.chdir(previousCwd);
    }
  });

  test("preserves unrelated settings", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, SETTINGS_PATH } = await loadModuleWithHome(home);

    await mkdir(join(home, ".pi", "agent"), { recursive: true });
    await writeFile(
      SETTINGS_PATH,
      `${JSON.stringify({ theme: "dark", compaction: { enabled: true }, defaultProvider: "old", defaultModel: "old" }, null, 2)}\n`,
      "utf8"
    );

    await persistStartupDefaults({ defaultProvider: "aura", defaultModel: "new-model" });

    const saved = JSON.parse(await readFile(SETTINGS_PATH, "utf8"));
    assert.equal(saved.theme, "dark");
    assert.deepEqual(saved.compaction, { enabled: true });
    assert.equal(saved.defaultProvider, "aura");
    assert.equal(saved.defaultModel, "new-model");
  });

  test("preserves unrelated local settings", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, LOCAL_SETTINGS_PATH } = await loadModuleWithHome(home);

    const projectDir = await mkdtemp(join(tmpdir(), "model-save-project-"));
    tempHomes.push(projectDir);
    const previousCwd = process.cwd();
    process.chdir(projectDir);

    try {
      await mkdir(join(projectDir, ".pi"), { recursive: true });
      await writeFile(
        LOCAL_SETTINGS_PATH(),
        `${JSON.stringify({ theme: "dark", thinking: "high", defaultProvider: "old", defaultModel: "old" }, null, 2)}\n`,
        "utf8"
      );

      await persistStartupDefaults({ defaultProvider: "aura", defaultModel: "new-local-model" }, LOCAL_SETTINGS_PATH());

      const saved = JSON.parse(await readFile(LOCAL_SETTINGS_PATH(), "utf8"));
      assert.equal(saved.theme, "dark");
      assert.equal(saved.thinking, "high");
      assert.equal(saved.defaultProvider, "aura");
      assert.equal(saved.defaultModel, "new-local-model");
    } finally {
      process.chdir(previousCwd);
    }
  });

  test("fails on invalid local settings json without overwriting the file", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, LOCAL_SETTINGS_PATH } = await loadModuleWithHome(home);

    const projectDir = await mkdtemp(join(tmpdir(), "model-save-project-"));
    tempHomes.push(projectDir);
    const previousCwd = process.cwd();
    process.chdir(projectDir);

    try {
      await mkdir(join(projectDir, ".pi"), { recursive: true });
      await writeFile(LOCAL_SETTINGS_PATH(), "{ not json\n", "utf8");

      await assert.rejects(
        persistStartupDefaults({ defaultProvider: "aura", defaultModel: "gpt-4o" }, LOCAL_SETTINGS_PATH()),
        /Unexpected token|Expected property name/
      );

      assert.equal(await readFile(LOCAL_SETTINGS_PATH(), "utf8"), "{ not json\n");
    } finally {
      process.chdir(previousCwd);
    }
  });

  test("resolves the local settings path from the current directory at call time", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, LOCAL_SETTINGS_PATH } = await loadModuleWithHome(home);

    const firstProjectDir = await mkdtemp(join(tmpdir(), "model-save-project-"));
    const secondProjectDir = await mkdtemp(join(tmpdir(), "model-save-project-"));
    tempHomes.push(firstProjectDir, secondProjectDir);
    const previousCwd = process.cwd();

    try {
      process.chdir(firstProjectDir);
      const firstPath = LOCAL_SETTINGS_PATH();
      process.chdir(secondProjectDir);
      const secondPath = LOCAL_SETTINGS_PATH();

      await persistStartupDefaults({ defaultProvider: "aura", defaultModel: "gpt-4.1" }, secondPath);

      await assert.rejects(readFile(firstPath, "utf8"), (error: unknown) => {
        return (error as NodeJS.ErrnoException).code === "ENOENT";
      });
      const saved = JSON.parse(await readFile(secondPath, "utf8"));
      assert.equal(saved.defaultProvider, "aura");
      assert.equal(saved.defaultModel, "gpt-4.1");
    } finally {
      process.chdir(previousCwd);
    }
  });

  test("fails on invalid settings json without overwriting the file", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { persistStartupDefaults, SETTINGS_PATH } = await loadModuleWithHome(home);

    await mkdir(join(home, ".pi", "agent"), { recursive: true });
    await writeFile(SETTINGS_PATH, "{ not json\n", "utf8");

    await assert.rejects(
      persistStartupDefaults({ defaultProvider: "aura", defaultModel: "gpt-4o" }),
      /Unexpected token|Expected property name/
    );

    assert.equal(await readFile(SETTINGS_PATH, "utf8"), "{ not json\n");
  });
});

describe("model selection helpers", () => {
  test("returns provider/model completions ranked for matching input", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { createModelCompletions } = await loadModuleWithHome(home);

    const completions = createModelCompletions(
      [
        { provider: "aura", id: "gpt-4o", name: "GPT-4o" },
        { provider: "openai", id: "gpt-5", name: "GPT-5" },
        { provider: "openai", id: "o3", name: "o3" }
      ],
      "gpt"
    );

    assert.deepEqual(completions, [
      { value: "aura/gpt-4o", label: "gpt-4o", description: "aura" },
      { value: "openai/gpt-5", label: "gpt-5", description: "openai" }
    ]);
  });

  test("resolves exact provider/model arguments without opening the picker", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { resolveModel } = await loadModuleWithHome(home);

    const resolved = resolveModel(
      [
        { provider: "aura", id: "gpt-4o", name: "GPT-4o" },
        { provider: "openai", id: "gpt-5", name: "GPT-5" }
      ],
      "openai/gpt-5"
    );

    assert.deepEqual(resolved, { provider: "openai", id: "gpt-5", name: "GPT-5" });
  });

  test("does not auto-resolve ambiguous short arguments", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { resolveModel } = await loadModuleWithHome(home);

    const resolved = resolveModel(
      [
        { provider: "aura", id: "gpt-4o", name: "GPT-4o" },
        { provider: "openai", id: "gpt-5", name: "GPT-5" }
      ],
      "gpt"
    );

    assert.equal(resolved, null);
  });

  test("registers slash-command completions backed by latest session models", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { default: registerModelSave } = await loadModuleWithHome(home);

    const handlers = new Map<string, (event: unknown, ctx: unknown) => void>();
    const commands: Array<{ name: string; options: { getArgumentCompletions?: (prefix: string) => unknown } }> = [];

    registerModelSave({
      on(event: string, handler: (event: unknown, ctx: unknown) => void) {
        handlers.set(event, handler);
        return () => {};
      },
      registerCommand(name: string, options: { getArgumentCompletions?: (prefix: string) => unknown }) {
        commands.push({ name, options });
      }
    } as never);

    const ctx = {
      modelRegistry: {
        getAll() {
          return Array.from({ length: 30 }, (_, index) => ({
            provider: index % 2 === 0 ? "aura" : "openai",
            id: `model-${String(index + 1).padStart(2, "0")}`,
            name: `Model ${String(index + 1).padStart(2, "0")}`
          }));
        }
      }
    };

    handlers.get("session_start")?.({}, ctx);

    const command = commands.find((entry) => entry.name === "model-save")?.options;
    assert.ok(command?.getArgumentCompletions);
    const completions = command.getArgumentCompletions?.("") as
      | Array<{ value: string; label: string; description: string }>
      | null
      | undefined;
    assert.ok(completions);
    assert.equal(completions.length, 10);
    assert.deepEqual(completions[0], { value: "aura/model-01", label: "model-01", description: "aura" });
    assert.deepEqual(completions[9], { value: "openai/model-10", label: "model-10", description: "openai" });
  });

  test("caps interactive picker options to 10 entries", async () => {
    const home = await mkdtemp(join(tmpdir(), "model-save-home-"));
    tempHomes.push(home);
    const { default: registerModelSave } = await loadModuleWithHome(home);

    let handler: ((args: string, ctx: ExtensionCommandContext) => Promise<void>) | undefined;
    const optionsSeen: string[][] = [];

    registerModelSave({
      on() {
        return () => {};
      },
      registerCommand(
        name: string,
        options: { handler?: (args: string, ctx: ExtensionCommandContext) => Promise<void> }
      ) {
        if (name === "model-save") {
          handler = options.handler;
        }
      }
    } as never);

    assert.ok(handler);

    await handler("", {
      mode: "tui",
      model: { provider: "aura", id: "model-01", name: "Model 01" },
      modelRegistry: {
        getAll() {
          return Array.from({ length: 30 }, (_, index) => ({
            provider: index % 2 === 0 ? "aura" : "openai",
            id: `model-${String(index + 1).padStart(2, "0")}`,
            name: `Model ${String(index + 1).padStart(2, "0")}`
          }));
        },
        find() {
          return undefined;
        }
      },
      ui: {
        async select(_title: string, options: string[]) {
          optionsSeen.push(options);
          return undefined;
        },
        notify() {}
      }
    } as never);

    assert.equal(optionsSeen.length, 1);
    assert.equal(optionsSeen[0].length, 10);
    assert.equal(optionsSeen[0][0], "Model 01 (aura/model-01)");
  });
});
