import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PirSystem } from "../target/types/pir_system";
import { expect } from "chai";
import { randomBytes } from "crypto";
import {
  RescueCipher,
  x25519,
  getMXEPublicKeyWithRetry,
  awaitComputationFinalization,
  getComputationAccAddress,
  getMXEAccAddress,
  getMempoolAccAddress,
  getExecutingPoolAccAddress,
  getCompDefAccAddress,
  getCompDefAccOffset,
  readKpJson,
  deserializeLE,
  awaitEvent,
  getArciumEnv,
  initPirQueryCompDef,
} from "@arcium-hq/client";
import * as os from "os";

describe("PIR System", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.PirSystem as Program<PirSystem>;
  const provider = anchor.getProvider();
  const arciumEnv = getArciumEnv();

  it("Performs PIR query!", async () => {
    const owner = readKpJson(`${os.homedir()}/.config/solana/id.json`);

    console.log("Initializing PIR computation definition");

    const initSig = await initPirQueryCompDef(program, owner, false);
    console.log("PIR computation definition initialized:", initSig);

    // Setup encryption
    const privateKey = x25519.utils.randomPrivateKey();
    const publicKey = x25519.getPublicKey(privateKey);
    const mxePublicKey = await getMXEPublicKeyWithRetry(
      provider as anchor.AnchorProvider,
      program.programId
    );

    const sharedSecret = x25519.getSharedSecret(privateKey, mxePublicKey);
    const cipher = new RescueCipher(sharedSecret);

    const queryId = BigInt(2);
    const nonce = randomBytes(16);
    const ciphertext = cipher.encrypt([queryId], nonce);

    const resultEventPromise = awaitEvent("pirResultEvent");
    const computationOffset = new anchor.BN(randomBytes(8), "hex");

    const queueSig = await program.methods
      .pirQuery(
        computationOffset,
        Array.from(ciphertext[0]),
        Array.from(publicKey),
        new anchor.BN(deserializeLE(nonce).toString())
      )
      .accountsPartial({
        payer: owner.publicKey,
        computationAccount: getComputationAccAddress(
          program.programId,
          computationOffset
        ),
        clusterAccount: arciumEnv.arciumClusterPubkey,
        mxeAccount: getMXEAccAddress(program.programId),
        mempoolAccount: getMempoolAccAddress(program.programId),
        executingPool: getExecutingPoolAccAddress(program.programId),
        compDefAccount: getCompDefAccAddress(
          program.programId,
          Buffer.from(getCompDefAccOffset("pir_query")).readUInt32LE()
        ),
      })
      .signers([owner])
      .rpc({ commitment: "confirmed" });

    console.log("PIR query submitted:", queueSig);

    const finalizeSig = await awaitComputationFinalization(
      provider.connection,
      computationOffset,
      program.programId,
      "confirmed"
    );

    console.log("Computation was finalized with sig: ", finalizeSig);

    const resultEvent = await resultEventPromise;
    const decryptedResult = cipher.decrypt(
      [resultEvent.result],
      resultEvent.nonce
    )[0];

    console.log("PIR query result:", decryptedResult);
    expect(decryptedResult).to.equal(BigInt(200));
  });
});
