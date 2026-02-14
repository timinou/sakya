import type { Options } from "@wdio/types";
import path from "path";

const isCI = !!process.env.CI;

export const config: Options.Testrunner = {
  runner: "local",
  autoCompileOpts: {
    tsNodeOpts: {
      project: "./tsconfig.json",
    },
  },
  specs: ["./e2e-tauri/**/*.spec.ts"],
  maxInstances: 1,
  capabilities: [
    {
      "tauri:options": {
        application: path.resolve(
          "./src-tauri/target/release/sakya",
        ),
      },
    } as any,
  ],
  logLevel: "warn",
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },
};
