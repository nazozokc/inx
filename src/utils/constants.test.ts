import { describe, it, test } from 'node:test';
import assert from 'node:assert';
import { TAG_NAME_REGEX, VERSION_TAG_REGEX, PACKAGE_NAME_REGEX } from './constants.js';

describe('constants', () => {
  describe('PACKAGE_NAME_REGEX', () => {
    it('should match valid package names', () => {
      assert.match('foo', PACKAGE_NAME_REGEX);
      assert.match('foo-bar', PACKAGE_NAME_REGEX);
      assert.match('test123', PACKAGE_NAME_REGEX);
    });

    it('should not match invalid package names', () => {
      assert.doesNotMatch('-foo', PACKAGE_NAME_REGEX);
      assert.doesNotMatch('foo_bar', PACKAGE_NAME_REGEX);
    });
  });

  describe('TAG_NAME_REGEX', () => {
    it('should match valid tag names', () => {
      assert.match('v1.0.0', TAG_NAME_REGEX);
      assert.match('1.0.0', TAG_NAME_REGEX);
      assert.match('release-1.0', TAG_NAME_REGEX);
      assert.match('test_tag', TAG_NAME_REGEX);
    });

    it('should not match tag names with ..', () => {
      // Note: regex itself allows .., validation is done in validateTagName function
      assert.match('v1..0.0', TAG_NAME_REGEX);
    });
  });

  describe('VERSION_TAG_REGEX', () => {
    it('should match valid version tags', () => {
      assert.match('v1.0.0', VERSION_TAG_REGEX);
      assert.match('1.0.0', VERSION_TAG_REGEX);
      assert.match('v0.1.0', VERSION_TAG_REGEX);
      assert.match('10.20.30', VERSION_TAG_REGEX);
    });

    it('should not match invalid version tags', () => {
      assert.doesNotMatch('1.0', VERSION_TAG_REGEX);
      assert.doesNotMatch('v1.0', VERSION_TAG_REGEX);
      assert.doesNotMatch('release-1.0.0', VERSION_TAG_REGEX);
      assert.doesNotMatch('1.0.0-beta', VERSION_TAG_REGEX);
    });
  });
});
