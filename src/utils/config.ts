import os from 'os';
import path from 'path';
import fs from 'fs/promises';
import { DEFAULT_CONFIG } from './constants.js';

export interface Config {
  registry: string;
}

export function getOxDir(): string {
  return path.join(os.homedir(), '.ox');
}

export function getConfigPath(): string {
  return path.join(getOxDir(), 'config.json');
}

export function getRegistryDir(): string {
  return path.join(getOxDir(), 'registry');
}

export function getPackagesDir(): string {
  return path.join(getOxDir(), 'packages');
}

export function validateRegistryUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' || parsed.protocol === 'ssh:';
  } catch {
    return false;
  }
}

export async function loadConfig(): Promise<Config> {
  const configPath = getConfigPath();
  try {
    const data = await fs.readFile(configPath, 'utf-8');
    const config = { ...DEFAULT_CONFIG, ...JSON.parse(data) };
    if (!validateRegistryUrl(config.registry)) {
      throw new Error(`Invalid registry URL: ${config.registry}. Only https: and ssh: protocols are allowed.`);
    }
    return config;
  } catch {
    return DEFAULT_CONFIG;
  }
}

export async function ensureOxDirs(): Promise<void> {
  await fs.mkdir(getOxDir(), { recursive: true });
  await fs.mkdir(getPackagesDir(), { recursive: true });
}