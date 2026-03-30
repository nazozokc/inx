import { Command } from 'commander';
import { listInstalledPackages, installPackage } from '../utils/packages.js';
import { updateRegistry, getTags, checkoutTag } from '../utils/registry.js';
import { VERSION_TAG_REGEX } from '../utils/constants.js';

function compareVersions(a: string, b: string): number {
  const parseVersion = (v: string) => {
    const cleaned = v.replace(/^v/, '').replace(/[^\d.]/g, '');
    const parts = cleaned.split('.').map(p => parseInt(p, 10) || 0);
    return parts;
  };
  
  const aParts = parseVersion(a);
  const bParts = parseVersion(b);
  
  for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
    const aVal = aParts[i] || 0;
    const bVal = bParts[i] || 0;
    if (aVal !== bVal) {
      return aVal - bVal;
    }
  }
  return 0;
}

export const upgrade = new Command()
  .name('upgrade')
  .description('Upgrade all installed packages to latest version')
  .action(async () => {
    try {
      await updateRegistry();

      const packages = await listInstalledPackages();
      if (packages.length === 0) {
        console.log('No packages installed');
        return;
      }

      const tags = await getTags();
      if (tags.length === 0) {
        console.log('No tags found in registry');
        return;
      }

      const validTags = tags.filter(tag => VERSION_TAG_REGEX.test(tag));
      if (validTags.length === 0) {
        console.log('No valid version tags found in registry');
        return;
      }

      const sortedTags = [...validTags].sort(compareVersions);
      const latestTag = sortedTags.pop()!;
      console.log(`Checking out tag: ${latestTag}`);
      await checkoutTag(latestTag);

      console.log(`Upgrading ${packages.length} package(s)...`);
      const results = await Promise.allSettled(
        packages.map(pkg => installPackage(pkg.name, true))
      );

      const failedIndices = results
        .map((result, index) => result.status === 'rejected' ? index : -1)
        .filter(i => i !== -1);

      if (failedIndices.length > 0) {
        for (const index of failedIndices) {
          console.error(`Failed to upgrade ${packages[index].name}`);
        }
        console.log('Upgrade completed with errors');
        process.exit(1);
      }
      console.log('Upgrade complete');
    } catch (error) {
      console.error(`Error: ${error}`);
      process.exit(1);
    }
  });