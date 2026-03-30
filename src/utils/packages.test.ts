import { describe, it, test } from 'node:test';
import assert from 'node:assert';
import { validatePackageName } from './packages.js';

describe('validatePackageName', () => {
  it('should accept valid package names', () => {
    assert.strictEqual(validatePackageName('foo'), true);
    assert.strictEqual(validatePackageName('foo-bar'), true);
    assert.strictEqual(validatePackageName('test123'), true);
    assert.strictEqual(validatePackageName('a'), true);
  });

  it('should reject package names starting with hyphen', () => {
    assert.strictEqual(validatePackageName('-foo'), false);
  });

  it('should reject package names with invalid characters', () => {
    assert.strictEqual(validatePackageName('foo_bar'), false);
    assert.strictEqual(validatePackageName('foo.bar'), false);
    assert.strictEqual(validatePackageName('foo bar'), false);
  });

  it('should reject package names exceeding max length', () => {
    const longName = 'a'.repeat(215);
    assert.strictEqual(validatePackageName(longName), false);
  });

  it('should reject empty string', () => {
    assert.strictEqual(validatePackageName(''), false);
  });
});
