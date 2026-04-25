import { renderVisitor as renderJavaScriptUmiVisitor } from "@codama/renderers-js-umi";
import { renderVisitor as renderRustVisitor } from "@codama/renderers-rust";

import {
  accountNode,
  arrayTypeNode,
  booleanTypeNode,
  bytesTypeNode,
  constantDiscriminatorNode,
  constantValueNode,
  createFromRoot,
  definedTypeLinkNode,
  definedTypeNode,
  instructionAccountNode,
  instructionArgumentNode,
  instructionNode,
  numberTypeNode,
  numberValueNode,
  prefixedCountNode,
  programNode,
  publicKeyTypeNode,
  publicKeyValueNode,
  rootNode,
  sizePrefixTypeNode,
  stringTypeNode,
  stringValueNode,
  structFieldTypeNode,
  structTypeNode,
} from "codama";

const root = rootNode(
  programNode({
    name: "solana-record-service",
    publicKey: "srsUi2TVUUCyGcZdopxJauk8ZBzgAaHHZCVUhm5ifPa",
    version: "1.1.0",
    accounts: [
      accountNode({
        name: "class",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(1)),
          ),
        ],
        data: structTypeNode([
          structFieldTypeNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(1),
            defaultValueStrategy: "omitted",
          }),
          structFieldTypeNode({ name: "authority", type: publicKeyTypeNode() }),
          structFieldTypeNode({
            name: "isPermissioned",
            type: booleanTypeNode(),
          }),
          structFieldTypeNode({ name: "isFrozen", type: booleanTypeNode() }),
          structFieldTypeNode({
            name: "name",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u8"),
            ),
          }),
          structFieldTypeNode({
            name: "metadata",
            type: stringTypeNode("utf8"),
          }),
        ]),
      }),
      accountNode({
        name: "record",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(2)),
          ),
        ],
        data: structTypeNode([
          structFieldTypeNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(2),
            defaultValueStrategy: "omitted",
          }),
          structFieldTypeNode({ name: "class", type: publicKeyTypeNode() }),
          structFieldTypeNode({
            name: "ownerType",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(0),
            defaultValueStrategy: "omitted",
          }),
          structFieldTypeNode({ name: "owner", type: publicKeyTypeNode() }),
          structFieldTypeNode({ name: "isFrozen", type: booleanTypeNode() }),
          structFieldTypeNode({ name: "expiry", type: numberTypeNode("i64") }),
          structFieldTypeNode({
            name: "seed",
            type: sizePrefixTypeNode(bytesTypeNode(), numberTypeNode("u8")),
          }),
          structFieldTypeNode({ name: "data", type: bytesTypeNode() }),
        ]),
      }),
    ],
    instructions: [
      instructionNode({
        name: "createClass",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(0)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(0),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "isPermissioned",
            type: booleanTypeNode(),
          }),
          instructionArgumentNode({
            name: "isFrozen",
            type: booleanTypeNode(),
          }),
          instructionArgumentNode({
            name: "name",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u8"),
            ),
          }),
          instructionArgumentNode({
            name: "metadata",
            type: stringTypeNode("utf8"),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: false,
            docs: ["Authority used to create a new class"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: ["Account that will pay for the class account"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: true,
            docs: ["New class account to be initialized"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to open our new class account"],
          }),
        ],
      }),
      instructionNode({
        name: "updateClassMetadata",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(1)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(1),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "metadata",
            type: stringTypeNode("utf8"),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Authority used to update a class"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: [
              "Account that will pay of get refunded for the class update",
            ],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: true,
            docs: ["Class account to be updated"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to extend our class account"],
          }),
        ],
      }),
      instructionNode({
        name: "updateClassAuthority",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(2)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(2),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "newAuthority",
            type: publicKeyTypeNode(),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Authority used to update a class"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: [
              "Account that will pay of get refunded for the class update",
            ],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: true,
            docs: ["Class account to be updated"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to extend our class account"],
          }),
        ],
      }),
      instructionNode({
        name: "freezeClass",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(3)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(3),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "isFrozen",
            type: booleanTypeNode(),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Authority used to freeze/thaw a class"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: true,
            docs: ["Class account to be frozen/thawed"],
          }),
        ],
      }),
      instructionNode({
        name: "createRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(4)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(4),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "expiration",
            type: numberTypeNode("i64"),
          }),
          instructionArgumentNode({
            name: "seed",
            type: sizePrefixTypeNode(bytesTypeNode(), numberTypeNode("u8")),
          }),
          instructionArgumentNode({ name: "data", type: bytesTypeNode() }),
        ],
        accounts: [
          instructionAccountNode({
            name: "owner",
            isSigner: true,
            isWritable: false,
            docs: ["Owner of the new record"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: ["Account that will pay for the record account"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: true,
            docs: ["Class account for the record to be created"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be created"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to create our record account"],
          }),
          instructionAccountNode({
            name: "authority",
            isOptional: true,
            isSigner: true,
            isWritable: false,
            docs: ["Optional authority for permissioned classes"],
          }),
        ],
      }),
      instructionNode({
        name: "createRecordTokenizable",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(4)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(4),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "expiration",
            type: numberTypeNode("i64"),
          }),
          instructionArgumentNode({
            name: "seed",
            type: sizePrefixTypeNode(bytesTypeNode(), numberTypeNode("u8")),
          }),
          instructionArgumentNode({
            name: "metadata",
            type: definedTypeLinkNode("metadata"),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "owner",
            isSigner: true,
            isWritable: false,
            docs: ["Owner of the new record"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: ["Account that will pay for the record account"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: true,
            docs: ["Class account for the record to be created"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be created"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to create our record account"],
          }),
          instructionAccountNode({
            name: "authority",
            isOptional: true,
            isSigner: true,
            isWritable: false,
            docs: ["Optional authority for permissioned classes"],
          }),
        ],
      }),
      instructionNode({
        name: "updateRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(5)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(5),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({ name: "data", type: bytesTypeNode() }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: [
              "Account that will pay of get refunded for the record update",
            ],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be updated"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to extend our record account"],
          }),
        ],
      }),
      instructionNode({
        name: "updateRecordTokenizable",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(5)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(5),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "metadata",
            type: definedTypeLinkNode("metadata"),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: [
              "Account that will pay of get refunded for the record update",
            ],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be updated"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to extend our record account"],
          }),
        ],
      }),
      instructionNode({
        name: "updateRecordExpiry",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(6)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(6),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "expiry",
            type: numberTypeNode("i64"),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: [
              "Account that will pay of get refunded for the record update",
            ],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be updated"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to extend our record account"],
          }),
        ],
      }),
      instructionNode({
        name: "transferRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(7)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(7),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "newOwner",
            type: publicKeyTypeNode(),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be updated"],
          }),
          instructionAccountNode({
            name: "class",
            isOptional: true,
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
        ],
      }),
      instructionNode({
        name: "deleteRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(8)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(8),
            defaultValueStrategy: "omitted",
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: ["Account that will get refunded for the record deletion"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be updated"],
          }),
          instructionAccountNode({
            name: "class",
            isOptional: true,
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
          instructionAccountNode({
            name: "token2022Program",
            isOptional: true,
            isSigner: false,
            isWritable: false,
            docs: ["Token2022 Program used to close the mint account"],
          }),
          instructionAccountNode({
            name: "mint",
            isOptional: true,
            isSigner: false,
            isWritable: true,
            docs: ["Mint account for the tokenized record"],
          }),
        ],
      }),
      instructionNode({
        name: "freezeRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(9)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(9),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "isFrozen",
            type: booleanTypeNode(),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account to be updated"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
        ],
      }),
      instructionNode({
        name: "mintTokenizedRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(10)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(10),
            defaultValueStrategy: "omitted",
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "owner",
            isSigner: false,
            isWritable: false,
            docs: ["Record owner"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: ["Account that will pay for the mint account"],
          }),
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: false,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account associated with the tokenized record"],
          }),
          instructionAccountNode({
            name: "mint",
            isSigner: false,
            isWritable: true,
            docs: ["Mint account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
          instructionAccountNode({
            name: "group",
            isSigner: false,
            isWritable: true,
            docs: ["Group account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "tokenAccount",
            isSigner: false,
            isWritable: true,
            docs: ["Token Account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "associatedTokenProgram",
            defaultValue: publicKeyValueNode(
              "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
              "associatedTokenProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["Associated Token Program used to create our token"],
          }),
          instructionAccountNode({
            name: "token2022",
            defaultValue: publicKeyValueNode(
              "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
              "token2022",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["Token2022 Program used to create our token"],
          }),
          instructionAccountNode({
            name: "systemProgram",
            defaultValue: publicKeyValueNode(
              "11111111111111111111111111111111",
              "systemProgram",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["System Program used to create our token"],
          }),
        ],
      }),
      instructionNode({
        name: "freezeTokenizedRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(11)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(11),
            defaultValueStrategy: "omitted",
          }),
          instructionArgumentNode({
            name: "isFrozen",
            type: booleanTypeNode(),
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: false,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "mint",
            isSigner: false,
            isWritable: false,
            docs: ["Mint account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "tokenAccount",
            isSigner: false,
            isWritable: true,
            docs: ["Token Account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: false,
            docs: ["Record account associated with the tokenized record"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
          instructionAccountNode({
            name: "token2022",
            defaultValue: publicKeyValueNode(
              "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
              "token2022",
            ),
            isSigner: false,
            isWritable: false,
            docs: [
              "Token2022 Program used to freeze/unfreeze the tokenized record",
            ],
          }),
        ],
      }),
      instructionNode({
        name: "transferTokenizedRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(12)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(12),
            defaultValueStrategy: "omitted",
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: false,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "mint",
            isSigner: false,
            isWritable: false,
            docs: ["Mint account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "tokenAccount",
            isSigner: false,
            isWritable: true,
            docs: ["Token Account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "newTokenAccount",
            isSigner: false,
            isWritable: true,
            docs: ["New Token Account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: false,
            docs: ["Record account associated with the tokenized record"],
          }),
          instructionAccountNode({
            name: "token2022",
            defaultValue: publicKeyValueNode(
              "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
              "token2022",
            ),
            isSigner: false,
            isWritable: false,
            docs: [
              "Token2022 Program used to freeze/unfreeze the tokenized record",
            ],
          }),
          instructionAccountNode({
            name: "class",
            isOptional: true,
            isSigner: false,
            isWritable: false,
            docs: ["Class account of the record"],
          }),
        ],
      }),
      instructionNode({
        name: "burnTokenizedRecord",
        discriminators: [
          constantDiscriminatorNode(
            constantValueNode(numberTypeNode("u8"), numberValueNode(13)),
          ),
        ],
        arguments: [
          instructionArgumentNode({
            name: "discriminator",
            type: numberTypeNode("u8"),
            defaultValue: numberValueNode(13),
            defaultValueStrategy: "omitted",
          }),
        ],
        accounts: [
          instructionAccountNode({
            name: "authority",
            isSigner: true,
            isWritable: true,
            docs: ["Record owner or class authority for permissioned classes"],
          }),
          instructionAccountNode({
            name: "payer",
            isSigner: true,
            isWritable: true,
            docs: [
              "Account that will get refunded for the tokenized record burn",
            ],
          }),
          instructionAccountNode({
            name: "mint",
            isSigner: false,
            isWritable: true,
            docs: ["Mint account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "tokenAccount",
            isSigner: false,
            isWritable: true,
            docs: ["Token Account for the tokenized record"],
          }),
          instructionAccountNode({
            name: "record",
            isSigner: false,
            isWritable: true,
            docs: ["Record account associated with the tokenized record"],
          }),
          instructionAccountNode({
            name: "token2022",
            defaultValue: publicKeyValueNode(
              "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
              "token2022",
            ),
            isSigner: false,
            isWritable: false,
            docs: ["Token2022 Program used to burn the tokenized record"],
          }),
          instructionAccountNode({
            name: "class",
            isSigner: false,
            isWritable: false,
            isOptional: true,
            docs: ["Class account of the record"],
          }),
        ],
      }),
    ],
    definedTypes: [
      definedTypeNode({
        name: "metadata",
        docs: "Token22 Metadata Extension compatible Metadata format",
        type: structTypeNode([
          structFieldTypeNode({
            name: "name",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u32"),
            ),
          }),
          structFieldTypeNode({
            name: "symbol",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u32"),
            ),
            defaultValue: stringValueNode("SRS"),
            defaultValueStrategy: "optional",
          }),
          structFieldTypeNode({
            name: "uri",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u32"),
            ),
          }),
          structFieldTypeNode({
            name: "additionalMetadata",
            type: arrayTypeNode(
              definedTypeLinkNode("additionalMetadata"),
              prefixedCountNode(numberTypeNode("u32")),
            ),
          }),
        ]),
      }),
      definedTypeNode({
        name: "additionalMetadata",
        docs: "Additional metadata for Token22 Metadata Extension compatible Metadata format",
        type: structTypeNode([
          structFieldTypeNode({
            name: "label",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u32"),
            ),
          }),
          structFieldTypeNode({
            name: "value",
            type: sizePrefixTypeNode(
              stringTypeNode("utf8"),
              numberTypeNode("u32"),
            ),
          }),
        ]),
      }),
    ],
  }),
);

const codama = createFromRoot(root);

codama.accept(renderJavaScriptUmiVisitor("sdk/ts/src", { formatCode: true }));
codama.accept(
  renderRustVisitor("sdk/rust", {
    formatCode: true,
  }),
);
