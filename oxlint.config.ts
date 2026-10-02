import solanaConfig from '@solana-config/oxc/oxlint';
import { defineConfig } from 'oxlint';

export default defineConfig({
    extends: [solanaConfig],
    ignorePatterns: ['**/dist/**', '**/generated/**', 'idl/**', 'target/**', '**/*.toml'],
    options: { typeAware: true },
    rules: { 'sort-keys': 'off' },
});
