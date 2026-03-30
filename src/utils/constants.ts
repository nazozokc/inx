/**
 * Default configuration values
 */
export const DEFAULT_CONFIG = {
  registry: 'https://github.com/nazozokc/ox',
};

/**
 * Regular expression for validating package names
 * - Must start with alphanumeric
 * - Can contain alphanumeric and hyphens
 * - Maximum length: 214 characters
 */
export const PACKAGE_NAME_REGEX = /^[a-zA-Z0-9][a-zA-Z0-9-]*$/;
export const PACKAGE_NAME_MAX_LENGTH = 214;

/**
 * Regular expression for validating tag names
 * - Only alphanumeric, dots, underscores, and hyphens
 * - Cannot contain '..' or start with '-'
 */
export const TAG_NAME_REGEX = /^[a-zA-Z0-9._-]+$/;

/**
 * Regular expression for validating version tags (e.g., v1.0.0, 1.0.0)
 */
export const VERSION_TAG_REGEX = /^v?\d+\.\d+\.\d+$/;
