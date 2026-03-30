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

export async function loadConfig(): Promise<Config> {
  const configPath = getConfigPath();
  try {
    const data = await fs.readFile(configPath, 'utf-8');
    return { ...DEFAULT_CONFIG, ...JSON.parse(data) };
  } catch {
    return DEFAULT_CONFIG;
  }
}

export async function ensureOxDirs(): Promise<void> {
  await fs.mkdir(getOxDir(), { recursive: true });
  await fs.mkdir(getPackagesDir(), { recursive: true });
}