import { renderJavaScriptVisitor, renderRustVisitor } from '@codama/renderers';
import { accountNode, booleanTypeNode, CODAMA_ERROR__UNRECOGNIZED_NUMBER_FORMAT, constantDiscriminatorNode, constantValueNode, createFromRoot, instructionAccountNode, instructionArgumentNode, instructionNode, numberTypeNode, numberValueNode, optionTypeNode, programNode, publicKeyTypeNode, publicKeyValueNode, rootNode, sizeDiscriminatorNode, sizePrefixTypeNode, STANDALONE_VALUE_NODE_KINDS, stringTypeNode, structFieldTypeNode, structTypeNode, TYPE_NODES } from "codama"

const root = rootNode(
    programNode({
        name: "srs",
        publicKey: "srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPaC",
        version: "1.0.0",
        accounts: [
            accountNode({
                name: "class",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(1)))
                ],
                data: structTypeNode([
                    structFieldTypeNode({ name: 'authority', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'isFrozen', type: booleanTypeNode() }),
                    structFieldTypeNode({ name: 'credentialAccount', type: optionTypeNode(publicKeyTypeNode()) }),
                    structFieldTypeNode({ 
                        name: 'name', 
                        type: sizePrefixTypeNode(stringTypeNode("utf8"), numberTypeNode("u8")),
                    }),
                    structFieldTypeNode({ name: 'metadata', type: stringTypeNode("utf8") }),
                ])
            })
        ],
        instructions: [
            instructionNode({
                name: "createClass",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(1)))
                ],
                arguments: [
                    instructionArgumentNode({ name: 'isPermissioned', type: booleanTypeNode() }),
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
                    })
                ]
            })
        ]
    })
)

const codama = createFromRoot(root)

codama.accept(renderJavaScriptVisitor('clients/js/src/generated'));
codama.accept(renderRustVisitor('clients/rust/src/generated'));