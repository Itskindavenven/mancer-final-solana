import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaCrowdfund } from "../target/types/solana_crowdfund";
import { assert } from "chai";

describe("solana-crowdfund", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaCrowdfund as Program<SolanaCrowdfund>;

  const creator = anchor.web3.Keypair.generate();
  const donor1 = anchor.web3.Keypair.generate();
  const donor2 = anchor.web3.Keypair.generate();
  const fakeVault = anchor.web3.Keypair.generate(); // For injection test

  const successfulCampaignId = new anchor.BN(1);
  const failedCampaignId = new anchor.BN(2);

  let successCampaignPda: anchor.web3.PublicKey;
  let successVaultPda: anchor.web3.PublicKey;
  let successContribution1Pda: anchor.web3.PublicKey;
  let successContribution2Pda: anchor.web3.PublicKey;

  let failedCampaignPda: anchor.web3.PublicKey;
  let failedVaultPda: anchor.web3.PublicKey;
  let failedContribution1Pda: anchor.web3.PublicKey;

  const delay = (ms: number) => new Promise((res) => setTimeout(res, ms));

  before(async () => {
    // Airdrop SOL
    const airdropSignatures = await Promise.all([
      provider.connection.requestAirdrop(creator.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(donor1.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(donor2.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
    ]);
    for (const sig of airdropSignatures) {
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        signature: sig,
        ...latestBlockhash,
      });
    }

    // Derive PDAs for Success Campaign
    [successCampaignPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("campaign"), creator.publicKey.toBuffer(), successfulCampaignId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [successVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), successCampaignPda.toBuffer()],
      program.programId
    );
    [successContribution1Pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contribution"), successCampaignPda.toBuffer(), donor1.publicKey.toBuffer()],
      program.programId
    );
    [successContribution2Pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contribution"), successCampaignPda.toBuffer(), donor2.publicKey.toBuffer()],
      program.programId
    );

    // Derive PDAs for Failed Campaign
    [failedCampaignPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("campaign"), creator.publicKey.toBuffer(), failedCampaignId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [failedVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), failedCampaignPda.toBuffer()],
      program.programId
    );
    [failedContribution1Pda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("contribution"), failedCampaignPda.toBuffer(), donor1.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("Campaign Creation & Contribution", () => {
    it("Creates a campaign successfully", async () => {
      const now = Math.floor(Date.now() / 1000);
      const deadline = new anchor.BN(now + 3); // 3 seconds alive
      const goal = new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL);

      await program.methods
        .createCampaign(successfulCampaignId, goal, deadline)
        .accounts({
          campaign: successCampaignPda,
          vault: successVaultPda,
          creator: creator.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([creator])
        .rpc();

      const campaignData = await program.account.campaign.fetch(successCampaignPda);
      assert.ok(campaignData.goal.eq(goal));
      assert.ok(campaignData.raised.eq(new anchor.BN(0)));
      assert.ok(campaignData.creator.equals(creator.publicKey));
      assert.ok(campaignData.campaignId.eq(successfulCampaignId));
    });

    it("Rejects 0 amount contribution", async () => {
      try {
        await program.methods
          .contribute(new anchor.BN(0))
          .accounts({
            campaign: successCampaignPda,
            contribution: successContribution1Pda,
            vault: successVaultPda,
            donor: donor1.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([donor1])
          .rpc();
        assert.fail("Should have rejected 0 amount");
      } catch (err: any) {
        assert.ok(err.message.includes("ZeroContribution") || err.message.includes("custom program error"));
      }
    });

    it("Rejects fake vault injection", async () => {
      try {
        await program.methods
          .contribute(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
          .accounts({
            campaign: successCampaignPda,
            contribution: successContribution1Pda,
            vault: fakeVault.publicKey, // FAKE VAULT
            donor: donor1.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([donor1])
          .rpc();
        assert.fail("Should have rejected fake vault");
      } catch (err: any) {
        assert.ok(err.message.includes("ConstraintSeeds") || err.message.includes("A seeds constraint was violated"));
      }
    });

    it("Accepts multiple contributions and prevents overflow implicitly (using checked_add)", async () => {
      const amount1 = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
      const amount2 = new anchor.BN(1.5 * anchor.web3.LAMPORTS_PER_SOL);

      await program.methods
        .contribute(amount1)
        .accounts({
          campaign: successCampaignPda,
          contribution: successContribution1Pda,
          vault: successVaultPda,
          donor: donor1.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([donor1])
        .rpc();

      await program.methods
        .contribute(amount2)
        .accounts({
          campaign: successCampaignPda,
          contribution: successContribution2Pda,
          vault: successVaultPda,
          donor: donor2.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([donor2])
        .rpc();

      const campaignData = await program.account.campaign.fetch(successCampaignPda);
      assert.ok(campaignData.raised.eq(new anchor.BN(2.5 * anchor.web3.LAMPORTS_PER_SOL)));
    });
  });

  describe("Withdrawal Attacks & Security", () => {
    it("Fails to withdraw before deadline", async () => {
      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: successCampaignPda,
            vault: successVaultPda,
            creator: creator.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([creator])
          .rpc();
        assert.fail("Should have failed");
      } catch (err: any) {
        assert.ok(err.message.includes("DeadlineNotReached"));
      }
    });

    it("Fails unauthorized withdrawal attempt (Not Creator)", async () => {
      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: successCampaignPda,
            vault: successVaultPda,
            creator: donor1.publicKey, // ATTACKER
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([donor1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err: any) {
        assert.ok(err.message.includes("ConstraintHasOne") || err.message.includes("Unauthorized"));
      }
    });

    it("Withdraws successfully after deadline", async () => {
      await delay(4000); // Wait for deadline to pass

      const initialCreatorBalance = await provider.connection.getBalance(creator.publicKey);

      await program.methods
        .withdraw()
        .accounts({
          campaign: successCampaignPda,
          vault: successVaultPda,
          creator: creator.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([creator])
        .rpc();

      const campaignData = await program.account.campaign.fetch(successCampaignPda);
      assert.ok(campaignData.claimed === true);

      const finalCreatorBalance = await provider.connection.getBalance(creator.publicKey);
      assert.ok(finalCreatorBalance > initialCreatorBalance);

      const vaultBalance = await provider.connection.getBalance(successVaultPda);
      assert.equal(vaultBalance, 0, "Vault should be drained");
    });

    it("Prevents Double Withdraw Attack", async () => {
      try {
        await program.methods
          .withdraw()
          .accounts({
            campaign: successCampaignPda,
            vault: successVaultPda,
            creator: creator.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([creator])
          .rpc();
        assert.fail("Should have failed double withdraw");
      } catch (err: any) {
        assert.ok(err.message.includes("AlreadyClaimed"));
      }
    });
  });

  describe("Refund Attacks & Security", () => {
    it("Creates a failed campaign", async () => {
      const now = Math.floor(Date.now() / 1000);
      const deadline = new anchor.BN(now + 2); // 2 seconds
      const goal = new anchor.BN(10 * anchor.web3.LAMPORTS_PER_SOL);

      await program.methods
        .createCampaign(failedCampaignId, goal, deadline)
        .accounts({
          campaign: failedCampaignPda,
          vault: failedVaultPda,
          creator: creator.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([creator])
        .rpc();

      await program.methods
        .contribute(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
        .accounts({
          campaign: failedCampaignPda,
          contribution: failedContribution1Pda,
          vault: failedVaultPda,
          donor: donor1.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([donor1])
        .rpc();
    });

    it("Fails refund if goal not missed (Deadline not reached)", async () => {
      try {
        await program.methods
          .refund()
          .accounts({
            campaign: failedCampaignPda,
            contribution: failedContribution1Pda,
            vault: failedVaultPda,
            donor: donor1.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([donor1])
          .rpc();
        assert.fail("Should have failed");
      } catch (err: any) {
        assert.ok(err.message.includes("DeadlineNotReached"));
      }
    });

    it("Refunds successfully and closes account", async () => {
      await delay(3000); // Pass deadline

      const initialDonorBalance = await provider.connection.getBalance(donor1.publicKey);

      await program.methods
        .refund()
        .accounts({
          campaign: failedCampaignPda,
          contribution: failedContribution1Pda,
          vault: failedVaultPda,
          donor: donor1.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([donor1])
        .rpc();

      const finalDonorBalance = await provider.connection.getBalance(donor1.publicKey);
      assert.ok(finalDonorBalance > initialDonorBalance);

      // Verify contribution account is closed
      try {
        await program.account.contribution.fetch(failedContribution1Pda);
        assert.fail("Account should be closed");
      } catch (err: any) {
        assert.ok(
          err.message.includes("Account does not exist") ||
          err.message.includes("AccountNotInitialized")
        );
      }
    });

    it("Prevents Multi-Refund Exploit (Double Refund)", async () => {
      try {
        await program.methods
          .refund()
          .accounts({
            campaign: failedCampaignPda,
            contribution: failedContribution1Pda, // Should fail since it's closed
            vault: failedVaultPda,
            donor: donor1.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          } as any)
          .signers([donor1])
          .rpc();
        assert.fail("Should have prevented double refund");
      } catch (err: any) {
        // Because the account is closed, Anchor will throw AccountNotInitialized/does not exist
        assert.ok(
          err.message.includes("AccountNotInitialized") ||
          err.message.includes("Account does not exist")
        );
      }
    });
  });
});
