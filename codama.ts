import { renderJavaScriptUmiVisitor, renderJavaScriptVisitor, renderRustVisitor } from '@codama/renderers';
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
            }),
            accountNode({
                name: "record",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(2)))
                ],
                data: structTypeNode([
                    structFieldTypeNode({ name: 'discriminator', type: numberTypeNode('u8'), defaultValue: numberValueNode(2), defaultValueStrategy: 'omitted' }),
                    structFieldTypeNode({ name: 'class', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'owner', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'isFrozen', type: booleanTypeNode() }),
                    structFieldTypeNode({ name: 'hasAuthorityExtension', type: booleanTypeNode() }),
                    structFieldTypeNode({ name: 'expiry', type: numberTypeNode("i64") }),
                    structFieldTypeNode({ name: 'name', type: sizePrefixTypeNode(stringTypeNode("utf8"), numberTypeNode("u8")) }),
                    structFieldTypeNode({ name: 'data', type: stringTypeNode("utf8") }),
                ])
            }),
            accountNode({
                name: "recordDelegate",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(3)))
                ],
                data: structTypeNode([
                    structFieldTypeNode({ name: 'discriminator', type: numberTypeNode('u8'), defaultValue: numberValueNode(3), defaultValueStrategy: 'omitted' }),
                    structFieldTypeNode({ name: 'record', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'updateAuthority', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'freezeAuthority', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'transferAuthority', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'burnAuthority', type: publicKeyTypeNode() }),
                    structFieldTypeNode({ name: 'authorityProgram', type: publicKeyTypeNode() }),
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
                        docs: ["Authority used to update a class"]
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
                        docs: ["System Program used to extend our class account"]
                    }),
                ]
            }),
            instructionNode({
                name: "freezeClass",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(2)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(2),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'isFrozen', type: booleanTypeNode() }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to freeze/thaw a class"]
                    }),
                    instructionAccountNode({
                        name: "class",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Class account to be frozen/thawed"]
                    })
                ]
            }),
            instructionNode({
                name: "createRecord",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(3)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(3),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ 
                        name: 'expiration', type: numberTypeNode("i64") 
                    }),
                    instructionArgumentNode({ name: 'name', type: sizePrefixTypeNode(stringTypeNode("utf8"), numberTypeNode("u8")) }),
                    instructionArgumentNode({ name: 'data', type: stringTypeNode("utf8") }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: "owner",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Owner of the new record"]
                    }),
                    instructionAccountNode({
                        name: "class",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Class account for the record to be created"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Record account to be created"]
                    }),
                    instructionAccountNode({
                        name: "systemProgram",
                        defaultValue: publicKeyValueNode('11111111111111111111111111111111', 'systemProgram'),
                        isSigner: false,
                        isWritable: false,
                        docs: ["System Program used to create our record account"]
                    }),
                    instructionAccountNode({
                        name: "authority",
                        isOptional: true,
                        isSigner: true,
                        isWritable: false,
                        docs: ["Optional authority for permissioned classes"]
                    }),
                ]
            }),
            instructionNode({
                name: "updateRecord",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(4)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(4),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'data', type: stringTypeNode("utf8") }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to update a record"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Record account to be updated"]
                    }),
                    instructionAccountNode({
                        name: "systemProgram",
                        defaultValue: publicKeyValueNode('11111111111111111111111111111111', 'systemProgram'),
                        isSigner: false,
                        isWritable: false,
                        docs: ["System Program used to extend our record account"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isOptional: true,
                        isSigner: true,
                        isWritable: false,
                        docs: ["Delegate signer for record account"]
                    }),
                ]
            }),
            instructionNode({
                name: "transferRecord",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(5)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(5),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'newOwner', type: publicKeyTypeNode() }),
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to update a record"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Record account to be updated"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isOptional: true,
                        isSigner: true,
                        isWritable: false,
                        docs: ["Delegate signer for record account"]
                    }),
                ]
            }),
            instructionNode({
                name: "deleteRecord",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(6)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(6),
                        defaultValueStrategy: 'omitted',
                    })
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to update a record"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Record account to be updated"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isOptional: true,
                        isSigner: true,
                        isWritable: false,
                        docs: ["Delegate signer for record account"]
                    }),
                ]
            }),
            instructionNode({
                name: "freezeRecord",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(7)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(7),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'isFrozen', type: booleanTypeNode() })
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to update a record"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Record account to be updated"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isOptional: true,
                        isSigner: true,
                        isWritable: false,
                        docs: ["Delegate signer for record account"]
                    }),
                ]
            }),
            instructionNode({
                name: "createRecordAuthorityDelegate",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(8)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(8),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'updateAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'freezeAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'transferAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'burnAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'authorityProgram', type: publicKeyTypeNode() })
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to create a delegate"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Record account to create delegate for"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Delegate for record account"]
                    }),
                    instructionAccountNode({
                        name: "systemProgram",
                        defaultValue: publicKeyValueNode('11111111111111111111111111111111', 'systemProgram'),
                        isSigner: false,
                        isWritable: false,
                        docs: ["System Program used to extend our record account"]
                    }),
                ]
            }),
            instructionNode({
                name: "updateRecordAuthorityDelegate",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(9)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(9),
                        defaultValueStrategy: 'omitted',
                    }),
                    instructionArgumentNode({ name: 'updateAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'freezeAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'transferAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'burnAuthority', type: publicKeyTypeNode() }),
                    instructionArgumentNode({ name: 'authorityProgram', type: publicKeyTypeNode() })
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to create a delegate"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: false,
                        docs: ["Record account to create delegate for"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Delegate for record account"]
                    })
                ]
            }),
            instructionNode({
                name: "deleteRecordAuthorityDelegate",
                discriminators: [
                    constantDiscriminatorNode(constantValueNode(numberTypeNode("u8"), numberValueNode(10)))
                ],
                arguments: [
                    instructionArgumentNode({
                        name: 'discriminator',
                        type: numberTypeNode('u8'),
                        defaultValue: numberValueNode(10),
                        defaultValueStrategy: 'omitted',
                    })
                ],
                accounts: [
                    instructionAccountNode({
                        name: "authority",
                        isSigner: true,
                        isWritable: true,
                        docs: ["Authority used to create a delegate"]
                    }),
                    instructionAccountNode({
                        name: "record",
                        isSigner: false,
                        isWritable: false,
                        docs: ["Record account to create delegate for"]
                    }),
                    instructionAccountNode({
                        name: "delegate",
                        isSigner: false,
                        isWritable: true,
                        docs: ["Delegate for record account"]
                    })
                ]
            })
        ]
    })
    // 11 => MintRecordToken::process(Context { accounts, data }),

)

const codama = createFromRoot(root)

codama.accept(renderJavaScriptUmiVisitor('sdk/ts/src', { formatCode: true }));
codama.accept(renderRustVisitor('sdk/rust/src/client', { crateFolder: 'sdk/rust/', formatCode: true }));