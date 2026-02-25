# 🚀 Solana Crowdfund Protocol (Production)

Deployment guide and manual verification steps for the secure protocol.

## 1. Prerequisites
- Rust & Cargo
- Solana CLI
- Anchor CLI (`@coral-xyz/anchor-cli`)
- Node.js & Yarn

## 2. Devnet Deployment Commands

1. **Configure Network:**
   ```bash
   solana config set --url devnet
   ```

2. **Generate Keypair & Airdrop:**
   ```bash
   solana-keygen new -o ~/.config/solana/id.json
   solana airdrop 2
   ```

3. **Build Program:**
   ```bash
   anchor build
   ```

4. **Verify Program ID:**
   ```bash
   solana address -k target/deploy/solana_crowdfund-keypair.json
   ```
   *Make sure this ID matches your `Anchor.toml` and `lib.rs` (`declare_id!`). Rebuild if it changed.*

5. **Deploy:**
   ```bash
   anchor deploy
   ```

## 3. Simulating Transactions
You can simulate transactions locally against an endpoint to review logs without spending fees:
```bash
solana account target/deploy/solana_crowdfund-keypair.json --url devnet
```

## 4. PDA Derivation Verification
For an external auditor, PDAs can be manually verified using the standard SHA-256 derivation over the specific derivation paths:
1. `campaign` PDA: `[ b"campaign", creator_pubkey, campaign_id (le_8_bytes), program_id ]`
2. `vault` PDA: `[ b"vault", campaign_pda, program_id ]`
3. `contribution` PDA: `[ b"contribution", campaign_pda, donor_pubkey, program_id ]`

## 5. Deployment Info
Example deployment transaction signature: 
`4vJ9JU1bJJE9vQQf4QG914S8GzH7bU4U4K1H8YJdK7JvA4Q5X9E8A4U5Y8K4J9H8X4Q5`
