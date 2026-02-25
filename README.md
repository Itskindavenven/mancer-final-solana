# 🚀 Solana Crowdfund Smart Contract

A production-ready, highly secure, and optimized crowdfunding smart contract built on the Solana blockchain using the Anchor framework.

This program allows creators to launch decentralized crowdfunding campaigns and enables donors to contribute funds securely. It is designed with robust security mitigations against common Web3 vulnerabilities such as Reentrancy attacks, Double-Withdrawals, and Fake Account Injections.

## ✨ Core Features

1. **Campaign Creation (`create_campaign`)**
   - Creators can initialize a campaign specifying a funding `goal` and a `deadline`.
   - Campaign data and Vault are securely managed via Program Derived Addresses (PDAs) to prevent unauthorized access.

2. **Secure Contributions (`contribute`)**
   - Donors can send SOL to the campaign's secure vault PDA.
   - Individual contributions are tracked accurately using a dedicated 1-to-1 `Contribution` PDA to prevent falsified refund claims.

3. **Goal-based Withdrawals (`withdraw`)**
   - The creator can only withdraw funds **IF and ONLY IF**:
     - The campaign deadline has passed.
     - The funding goal has been met.
   - Protected against double-claim exploits using state flags.

4. **Guaranteed Refunds (`refund`)**
   - If the campaign fails to reach its goal by the deadline, donors can autonomously claim their refunds.
   - The refund system uses an on-chain verification mechanism reading exactly from the `Contribution` PDA, rather than trusting user inputs, making vault-draining impossible.
   - Contribution PDAs are closed upon refund to return rent and natively prevent double-refund attacks.

## 🛡️ Security Architecture

The Smart Contract has been audited iteratively and includes the following security guarantees:
- **Checks-Effects-Interactions Pattern:** All state variables (e.g., `claimed`, `refunded`) are mutated *before* cross-program invocations (CPI) are made to transfer lamports.
- **Strict PDA Constraints:** Strong seed validations (`seeds = [...]`, `bump`) ensure that only legitimate, program-generated Vaults and Contribution accounts are interacted with.
- **Checked Math:** All arithmetic operations utilize `checked_add` and `checked_sub` to prevent integer overflow and underflow vulnerabilities.
- **Account Closure (`close = donor`):** Once a refund is processed, the state account is wiped and rent is returned instantly, perfectly mitigating re-entrancy without heavy logic.

---

## 💡 Planned Innovations (Roadmap)

To elevate this smart contract from a standard platform to a **Next-Gen Web3 Launchpad**, the following innovations are planned for the upcoming version (V2):

### 1. 🔐 Milestone-based Withdrawals (Voting Protocol)
Currently, a successful campaign allows the creator to withdraw 100% of the funds immediately. To protect investors from rug-pulls and ensure creator accountability:
- **Phase 1 (Initial Yield):** Creators can only withdraw an initial percentage (e.g., 30%) upon campaign success.
- **Phase 2+ (Milestones):** For the remaining funds, the creator must submit a `ProposeMilestone` transaction.
- **DAO Voting:** Donors (holding `Contribution` states) will call a `vote_milestone` instruction. Only if >50% of the pooled vote approves, will the next tranche of the Vault be unlocked.

### 2. 🏅 NFT Supporter Badges (Automated Minting)
Integrating **Metaplex Token Metadata** directly into the `contribute` instruction. 
- When a donor contributes above a certain tier (e.g., > 5 SOL), the smart contract atomically mints an exclusive "Gold Supporter" NFT directly to their wallet via CPI. 
- This NFT can be used for token-gated community access (e.g., Discord) or future airdrops by the creator.

### 3. 💸 Yield-Bearing Idle Vault
Integrating with Solana DeFi primitives (like Solend/Marginfi or Jito Stake Pool).
- While the campaign is running (before deadline), the locked funds in the Vault are automatically staked/lent.
- If the campaign succeeds, the creator receives the Goal + Yield. If the campaign fails, donors get exactly their principal back, while the yield can be distributed proportionally as a "reward for participating", making failed campaigns a *No-Loss Lottery* for donors.

---

## 🛠️ Testing

The project includes an exhaustive TypeScript test suite targeting exact vulnerability vectors.

```bash
# Run local testnet node and execute entire Mocha test suite
anchor test
```

Tested Scenarios Include:
- Successful campaign lifecycle bindings.
- Zero-amount contribution rejection.
- Fake Vault Injection rejection (Constraint violation).
- Math overflow prevention implicit assertions.
- Unauthorized withdrawal rejections.
- Double-withdrawal and Double-refund mathematical preventions.
- Account closure lifecycle checks.

## 📝 License
MIT License. Created for the Solana ecosystem.
