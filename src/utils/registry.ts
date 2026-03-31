import { simpleGit, SimpleGit } from 'simple-git';
import fs from 'fs/promises';
import { getRegistryDir, loadConfig, ensureOxDirs } from './config.js';

function getGit(workingDir?: string): SimpleGit {
  return simpleGit(workingDir, { timeout: { block: 30000 } });
}

export async function initRegistry(): Promise<void> {
  const registryDir = getRegistryDir();
  const config = await loadConfig();

  await ensureOxDirs();

  let isRepo = false;
  try {
    const git = getGit(registryDir);
    isRepo = await git.checkIsRepo();
  } catch {
    isRepo = false;
  }

  if (isRepo) {
    console.log('Updating registry...');
    const git = getGit(registryDir);
    await git.pull();
  } else {
    console.log('Cloning registry...');
    const repoUrl = config.registry.endsWith('.git') 
      ? config.registry 
      : `${config.registry}.git`;
    try {
      await fs.rm(registryDir, { recursive: true, force: true });
    } catch {
      // Directory does not exist, ignore
    }
    const git = getGit();
    await git.clone(repoUrl, registryDir);
  }
}

export async function updateRegistry(): Promise<void> {
  const registryDir = getRegistryDir();

  try {
    await fs.access(registryDir);
  } catch {
    await initRegistry();
    return;
  }

  const git = getGit(registryDir);
  try {
    const isRepo = await git.checkIsRepo();
    if (!isRepo) {
      await initRegistry();
      return;
    }
    console.log('Pulling latest changes...');
    await git.pull();
  } catch (error) {
    throw new Error(`Failed to update registry: ${error}`, { cause: error });
  }
}

export async function getTags(): Promise<string[]> {
  const registryDir = getRegistryDir();
  const git = getGit(registryDir);

  try {
    const tags = await git.tags();
    return tags.all;
  } catch {
    return [];
  }
}

import { TAG_NAME_REGEX } from './constants.js';

function validateTagName(tag: string): boolean {
  return TAG_NAME_REGEX.test(tag) && !tag.includes('..') && !tag.startsWith('-');
}

export async function checkoutTag(tag: string): Promise<void> {
  if (!validateTagName(tag)) {
    throw new Error(`Invalid tag name '${tag}'`);
  }
  
  const registryDir = getRegistryDir();
  const git = getGit(registryDir);

  try {
    await git.checkout(tag);
  } catch (error) {
    throw new Error(`Failed to checkout tag ${tag}: ${error}`, { cause: error });
  }
}