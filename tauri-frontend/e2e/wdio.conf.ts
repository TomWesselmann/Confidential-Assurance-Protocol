/**
 * WebdriverIO Configuration for Tauri E2E Tests
 *
 * @description E2E testing configuration for CAP Desktop Proofer
 * Uses tauri-driver for native WebDriver protocol communication
 */

import type { Options } from '@wdio/types';
import { spawn, ChildProcess } from 'child_process';
import path from 'path';

// Tauri driver process
let tauriDriver: ChildProcess | null = null;

export const config: Options.Testrunner = {
  // ====================
  // Runner Configuration
  // ====================
  runner: 'local',
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      transpileOnly: true,
      project: './e2e/tsconfig.json',
    },
  },

  // ==================
  // Specify Test Files
  // ==================
  specs: ['./e2e/specs/**/*.e2e.ts'],
  exclude: [],

  // ============
  // Capabilities
  // ============
  maxInstances: 1, // Tauri apps should run one at a time
  capabilities: [
    {
      'tauri:options': {
        application: '../src-tauri/target/release/cap-desktop-proofer',
      },
    },
  ],

  // ===================
  // Test Configurations
  // ===================
  logLevel: 'info',
  bail: 0,
  baseUrl: '',
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // =============
  // Test Framework
  // =============
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // =====
  // Hooks
  // =====

  /**
   * Start tauri-driver before tests
   */
  onPrepare: async function () {
    console.log('Starting tauri-driver...');

    tauriDriver = spawn('tauri-driver', [], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    tauriDriver.stdout?.on('data', (data: Buffer) => {
      console.log(`[tauri-driver] ${data.toString().trim()}`);
    });

    tauriDriver.stderr?.on('data', (data: Buffer) => {
      console.error(`[tauri-driver error] ${data.toString().trim()}`);
    });

    // Wait for driver to be ready
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log('tauri-driver started');
  },

  /**
   * Stop tauri-driver after tests
   */
  onComplete: async function () {
    if (tauriDriver) {
      console.log('Stopping tauri-driver...');
      tauriDriver.kill();
      tauriDriver = null;
    }
  },

  /**
   * Before each test file
   */
  before: async function () {
    // Add custom commands if needed
  },

  /**
   * After each test
   */
  afterTest: async function (test, context, { error }) {
    if (error) {
      // Take screenshot on failure
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const screenshotPath = `./e2e/screenshots/${test.title}-${timestamp}.png`;
      await browser.saveScreenshot(screenshotPath);
      console.log(`Screenshot saved: ${screenshotPath}`);
    }
  },
};

export default config;
