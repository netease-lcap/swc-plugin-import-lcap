import { expect, test, describe } from "vitest";
import { transform } from "@swc/core";
import path from "node:path";
import url from "node:url";
import fs from "node:fs/promises";

const pluginName = "lcap_swc_plugin_import.wasm";

const transformCode = async (
  code: string,
  options: Record<string, unknown>,
) => {
  return transform(code, {
    jsc: {
      parser: {
        syntax: "ecmascript",
      },
      experimental: {
        plugins: [
          [
            path.join(
              path.dirname(url.fileURLToPath(import.meta.url)),
              "..",
              pluginName,
            ),
            options,
          ],
        ],
      },
    },
    filename: "test.js",
  });
};

async function walkDir(
  dir: URL,
  callback: (
    dir: string,
    input: string,
    output: string,
    config: Record<string, unknown>,
  ) => Promise<void>,
) {
  const dirs = await fs.readdir(dir);
  const baseDir = url.fileURLToPath(dir);

  for (const dir of dirs) {
    const inputFilePath = path.join(baseDir, dir, "input.js");
    const outputFilePath = path.join(baseDir, dir, "output.js");
    const configFilePath = path.join(baseDir, dir, "config.json");


    try {
      const input = await fs.readFile(inputFilePath, "utf-8");
      const output = await fs.readFile(outputFilePath, "utf-8");

      const config = await fs.readFile(configFilePath, "utf-8").then(
        (json) => {
          return JSON.parse(json);
        },
        (_) => undefined,
      );


      await callback(dir, input, output, config);
    } catch (e) {
      console.log(e);
    }
  }
}

describe("Should load transform-imports wasm plugin correctly", async () => {
  await walkDir(
    new URL("./fixture", import.meta.url),
    async (dir, input, output, config) => {
      await test(`Should transform ${dir} correctly`, async () => {
        const { code } = await transformCode(input, config);
        expect(code).toMatch(output);
      });
    },
  );
});