import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import {
    assertIsNode,
    bottomUpTransformerVisitor,
    createFromJson,
    definedTypeLinkNode,
    instructionArgumentNode,
    type InstructionNode,
} from 'codama';

type Codama = ReturnType<typeof createFromJson>;

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const projectRoot = path.join(__dirname, '..');
const idlPath = path.join(projectRoot, 'idl', 'solana_record_service.json');
const rustClientsDir = path.join(projectRoot, 'clients', 'rust');
const typescriptClientsDir = path.join(projectRoot, 'clients', 'typescript');

const readIdl = () => createFromJson(fs.readFileSync(idlPath, 'utf-8'));

// A tokenizable record stores Token-2022 metadata as its data, which `MintTokenizedRecord`
// later reads in place. The program cannot tell the two encodings apart, so the tokenizable
// variants share the discriminator of the instruction they mirror and only retype `data`.
const TOKENIZABLE_VARIANTS: Record<string, { name: string; docs: string[] }> = {
    createRecord: {
        name: 'createRecordTokenizable',
        docs: ['Create a record whose data is Token-2022 metadata, so it can later be minted as a token.'],
    },
    updateRecord: {
        name: 'updateRecordTokenizable',
        docs: ['Replace the data of a record with Token-2022 metadata, so it can later be minted as a token.'],
    },
};

function shapeProgram(codama: Codama) {
    codama.update(
        bottomUpTransformerVisitor([
            {
                select: '[programNode]',
                transform: node => {
                    assertIsNode(node, 'programNode');

                    const tokenizable = node.instructions
                        .filter(instruction => instruction.name in TOKENIZABLE_VARIANTS)
                        .map((instruction): InstructionNode => ({
                            ...instruction,
                            ...TOKENIZABLE_VARIANTS[instruction.name],
                            arguments: instruction.arguments.map(argument =>
                                argument.name === 'data'
                                    ? instructionArgumentNode({
                                          name: 'metadata',
                                          type: definedTypeLinkNode('metadata'),
                                      })
                                    : argument,
                            ),
                        }));

                    return {
                        ...node,
                        // `Mint` and `Group` in `program/src/constants.rs` exist only to carry their
                        // PDA seeds. Their PDA helpers are worth generating; decoders for an empty
                        // account are not.
                        accounts: node.accounts.filter(account => (account.data.fields ?? []).length > 0),
                        instructions: [...node.instructions, ...tokenizable],
                    };
                },
            },
        ]),
    );
}

const rustCodama = readIdl();
shapeProgram(rustCodama);
rustCodama.accept(
    renderRustVisitor(rustClientsDir, {
        anchorTraits: false,
        deleteFolderBeforeRendering: true,
        formatCode: true,
        generatedFolder: 'src/generated',
        traitOptions: {
            baseDefaults: ['borsh::BorshSerialize', 'borsh::BorshDeserialize', 'Clone', 'Debug', 'Eq', 'PartialEq'],
        },
    }),
);

const tsCodama = readIdl();
shapeProgram(tsCodama);

// The renderer takes the package folder, writes to its src/generated, and syncs
// the dependency ranges below into clients/typescript/package.json on every run,
// so bumping kit means editing them here rather than in the manifest.
void tsCodama.accept(
    renderJavaScriptVisitor(typescriptClientsDir, {
        deleteFolderBeforeRendering: true,
        // `@solana/kit` re-exports the program client core helpers on a subpath.
        // Importing them from there keeps kit as the client's only external package.
        dependencyMap: {
            solanaProgramClientCore: '@solana/kit/program-client-core',
        },
        dependencyVersions: {
            '@solana/kit': '^8.0.0',
        },
        formatCode: true,
        prettierOptions: {
            arrowParens: 'always',
            printWidth: 80,
            semi: true,
            singleQuote: true,
            tabWidth: 2,
            trailingComma: 'es5',
            useTabs: false,
        },
    }),
);
