import { renderJavaScriptVisitor, renderRustVisitor } from '@codama/renderers';
import { accountNode, booleanTypeNode, constantDiscriminatorNode, constantValueNode, createFromRoot, instructionAccountNode, instructionArgumentNode, instructionNode, numberTypeNode, numberValueNode, optionTypeNode, programNode, publicKeyTypeNode, publicKeyValueNode, rootNode, sizeDiscriminatorNode, sizePrefixTypeNode, stringTypeNode, structFieldTypeNode, structTypeNode } from "codama"

const root = rootNode(
    programNode({
        name: "solana-record-service",
        publicKey: "srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPa",
        version: "1.0.0",
        accounts: [
            accountNode({
                name: "class",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(1)))
                ],
                data: structTypeNode([
                    structFieldTypeNode({ name: 'discriminator', type: numberTypeNode('u8'), defaultValue: numberValueNode(1), defaultValueStrategy: 'omitted' }),
                    structFieldTypeNode({ name: 'authority', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'isPermissioned', type: booleanTypeNode() }),
                    structFieldTypeNode({ name: 'isFrozen', type: booleanTypeNode() }),
                    structFieldTypeNode({ name: 'name', type: sizePrefixTypeNode(stringTypeNode("utf8"), numberTypeNode("u8")) }),
                    structFieldTypeNode({ name: 'metadata', type: stringTypeNode("utf8") }),
                ])
            })
        ],
        instructions: [
            instructionNode({
                name: "createClass",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(0)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(0),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'isPermissioned', type: booleanTypeNode() }),
                    instructionArgumentNode({ name: 'isFrozen', type: booleanTypeNode() }),
                    instructionArgumentNode({ name: 'name', type: sizePrefixTypeNode(stringTypeNode("utf8"), numberTypeNode("u8")) }),
                    instructionArgumentNode({ name: 'metadata', type: stringTypeNode("utf8") }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to create a new class"]
                    }),
                    instructionAccountNode({
                        name: "class",
                        isSigner: false,
                        isWritable: true,
                        docs: ["New class account to be initialized"]
                    }),
                    instructionAccountNode({
                        name: "systemProgram",
                        defaultValue: publicKeyValueNode('11111111111111111111111111111111', 'systemProgram'),
                        isSigner: false,
                        isWritable: false,
                        docs: ["System Program used to open our new class account"]
                    }),
                ]
            }),
            instructionNode({
                name: "updateClassMetadata",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(1)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(1),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'metadata', type: stringTypeNode("utf8") }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to create a new class"]
                    }),
                    instructionAccountNode({
                        name: "class",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Class account to be updated"]
                    }),
                    instructionAccountNode({
                        name: "systemProgram",
                        defaultValue: publicKeyValueNode('11111111111111111111111111111111', 'systemProgram'),
                        isSigner: false,
                        isWritable: false,
                        docs: ["System Program used to open our new class account"]
                    }),
                ]
            })
        ]
    })
)

const codama = createFromRoot(root)

codama.accept(renderJavaScriptVisitor('sdk/ts/src'));
codama.accept(renderRustVisitor('sdk/rust/src/client'));