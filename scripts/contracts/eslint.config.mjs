export default [
  {
    files: ['*.mjs'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { Buffer: 'readonly', process: 'readonly', URL: 'readonly' },
    },
    rules: {
      'no-undef': 'error',
      'no-unused-vars': 'error',
      'no-unreachable': 'error',
      'no-dupe-keys': 'error',
      'no-dupe-args': 'error',
      'no-constant-condition': 'error',
      'no-unexpected-multiline': 'error',
      eqeqeq: ['error', 'always', { null: 'ignore' }],
    },
  },
];
