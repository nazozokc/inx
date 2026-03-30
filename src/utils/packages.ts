import fs from 'fs/promises';
import path from 'path';
import { getPackagesDir, getRegistryDir, ensureOxDirs } from './config.js';
import { PACKAGE_NAME_REGEX, PACKAGE_NAME_MAX_LENGTH } from './constants.js';

export function validatePackageName(name: string): boolean {
  return PACKAGE_NAME_REGEX.test(name) && name.length <= PACKAGE_NAME_MAX_LENGTH;
}

export interface InstalledPackage {
  name: string;
  version: string;
  path: string;
}

export interface PackageJson {
  name: string;
  version: string;
  description?: string;
}

export async function listInstalledPackages(): Promise<InstalledPackage[]> {
  const packagesDir = getPackagesDir();
  const packages: InstalledPackage[] = [];

  try {
    const entries = await fs.readdir(packagesDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory()) {
        const pkgJsonPath = path.join(packagesDir, entry.name, 'package.json');
        try {
          const data = await fs.readFile(pkgJsonPath, 'utf-8');
          const pkg: PackageJson = JSON.parse(data);
          if (!pkg.name || !pkg.version) {
            console.warn(`Warning: Invalid package.json in '${entry.name}': missing name or version`);
            continue;
          }
          packages.push({
            name: pkg.name,
            version: pkg.version,
            path: path.join(packagesDir, entry.name),
          });
        } catch (error) {
          console.warn(`Warning: Failed to read package.json in '${entry.name}': ${error}`);
        }
      }
    }
  } catch {
    // No packages installed yet
  }

  return packages;
}

export async function getAvailablePackages(): Promise<string[]> {
  const registryDir = getRegistryDir();
  const packagesDir = path.join(registryDir, 'packages');
  const packages: string[] = [];

  try {
    const entries = await fs.readdir(packagesDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory()) {
        packages.push(entry.name);
      }
    }
  } catch {
    // Registry not initialized
  }

  return packages;
}

async function validateNoSymlinks(dirPath: string): Promise<boolean> {
  const resolvedBase = await fs.realpath(dirPath);

  const entries = await fs.readdir(dirPath, { withFileTypes: true });
  for (const entry of entries) {
    const entryPath = path.join(dirPath, entry.name);
    if (entry.isSymbolicLink()) {
      const resolved = await fs.realpath(entryPath);
      if (!resolved.startsWith(resolvedBase + path.sep)) {
        return false;
      }
    }
    if (entry.isDirectory() && !entry.isSymbolicLink()) {
      if (!(await validateNoSymlinks(entryPath))) {
        return false;
      }
    }
  }
  return true;
}

export async function installPackage(name: string, force = false): Promise<void> {
  if (!validatePackageName(name)) {
    throw new Error(`Invalid package name '${name}'. Use only letters, numbers, and hyphens.`);
  }

  const registryDir = getRegistryDir();
  const sourceDir = path.join(registryDir, 'packages', name);
  const destDir = path.join(getPackagesDir(), name);

  try {
    await fs.access(sourceDir);
  } catch {
    throw new Error(`Package '${name}' not found in registry`);
  }

  if (!(await validateNoSymlinks(sourceDir))) {
    throw new Error(`Package '${name}' contains invalid symbolic links`);
  }

  await ensureOxDirs();

  const destExists = await fs.access(destDir).then(() => true).catch(() => false);

  if (destExists) {
    if (!force) {
      console.log(`Package '${name}' is already installed`);
      return;
    }
    await fs.rm(destDir, { recursive: true, force: true });
  }

  try {
    await fs.cp(sourceDir, destDir, { recursive: true });
    console.log(`Installed '${name}'`);
  } catch (error) {
    try {
      await fs.rm(destDir, { recursive: true, force: true });
    } catch (rmError) {
      console.warn(`Warning: Failed to clean up '${destDir}': ${rmError}`);
    }
    throw new Error(`Failed to install '${name}': ${error}`);
  }
}

export async function uninstallPackage(name: string): Promise<void> {
  const destDir = path.join(getPackagesDir(), name);

  try {
    await fs.access(destDir);
  } catch {
    throw new Error(`Package '${name}' is not installed`);
  }

  await fs.rm(destDir, { recursive: true });
  console.log(`Uninstalled '${name}'`);
}