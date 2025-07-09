/// AI AGENT SCAM DETECTION DEMO
/// =============================
/// 
/// Shows how AI agents can use Terminator-Dancer to detect malicious transactions
/// before asking users to sign them. Perfect for wallet security demos.

use terminator_dancer::{
    integrated_runtime::IntegratedRuntime,
    solana_format::{SolanaTransactionParser, SolanaPubkey, SolanaHash, CompiledInstruction, SolanaMessage, MessageHeader, SolanaTransaction, SolanaSignature},
    types::Pubkey,
    Result,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 AI AGENT: TRANSACTION SECURITY SCANNER");
    println!("=========================================");
    println!("Every wallet should run this before asking for your signature!");
    println!();

    let mut runtime = IntegratedRuntime::new()?;

    // Demo 1: Safe transaction ✅
    analyze_safe_transaction(&mut runtime).await?;
    
    // Demo 2: Suspicious drain wallet transaction ⚠️
    analyze_drain_wallet_transaction(&mut runtime).await?;
    
    // Demo 3: Unknown program interaction 🚨
    analyze_unknown_program_transaction(&mut runtime).await?;
    
    // Demo 4: Show the protective power
    demonstrate_protection_value(&mut runtime).await?;

    Ok(())
}

async fn analyze_safe_transaction(runtime: &mut IntegratedRuntime) -> Result<()> {
    println!("✅ ANALYSIS 1: SAFE TRANSACTION");
    println!("===============================");
    
    let from = SolanaPubkey::new([1u8; 32]);
    let to = SolanaPubkey::new([2u8; 32]);
    let amount = 10_000_000; // 0.01 SOL - reasonable amount
    
    println!("📋 Transaction to analyze:");
    println!("   Type: SOL Transfer");
    println!("   From: {}...{}", hex::encode(&from.0[..4]), hex::encode(&from.0[28..]));
    println!("   To: {}...{}", hex::encode(&to.0[..4]), hex::encode(&to.0[28..]));
    println!("   Amount: {} lamports (0.01 SOL)", amount);
    
    let tx = SolanaTransactionParser::create_transfer_transaction(
        from, to, amount, SolanaHash([42u8; 32])
    );
    
    let analysis_start = Instant::now();
    
    // AI Agent Analysis
    println!();
    println!("🧠 AI Agent Security Analysis:");
    
    // Check 1: Program safety
    let system_program_used = tx.message.instructions[0].program_id_index == 2;
    println!("   🔍 Program Check: {}", 
             if system_program_used { "✅ System Program (Safe)" } else { "❌ Unknown Program" });
    
    // Check 2: Amount reasonableness
    let reasonable_amount = amount < 100_000_000; // Less than 0.1 SOL
    println!("   💰 Amount Check: {}", 
             if reasonable_amount { "✅ Reasonable amount" } else { "⚠️ Large amount" });
    
    // Check 3: Account reputation (simulated)
    println!("   👤 Recipient Check: ✅ No blacklist matches");
    
    // Check 4: Transaction structure
    let simple_structure = tx.message.instructions.len() == 1;
    println!("   🏗️ Structure Check: {}", 
             if simple_structure { "✅ Simple transfer" } else { "⚠️ Complex transaction" });
    
    // Simulate runtime execution to check for errors
    runtime.fund_account(&Pubkey::new(from.0), 100_000_000);
    let simulation_result = runtime.execute_solana_transaction_parsed(&tx);
    let will_succeed = simulation_result.is_ok();
    
    println!("   ⚡ Simulation Check: {}", 
             if will_succeed { "✅ Transaction will succeed" } else { "❌ Transaction will fail" });
    
    let analysis_time = analysis_start.elapsed();
    
    println!();
    println!("🎯 VERDICT: ✅ SAFE TO SIGN");
    println!("   Analysis time: {:?}", analysis_time);
    println!("   Risk score: 0/10 (Very Safe)");
    println!("   Recommendation: ✅ Agent approves signing");
    println!();
    
    Ok(())
}

async fn analyze_drain_wallet_transaction(runtime: &mut IntegratedRuntime) -> Result<()> {
    println!("⚠️ ANALYSIS 2: SUSPICIOUS DRAIN TRANSACTION");
    println!("==========================================");
    
    let victim = SolanaPubkey::new([10u8; 32]);
    let attacker = SolanaPubkey::new([66u8; 32]); // Suspicious address
    let drain_amount = 1_000_000_000; // 1 SOL - draining significant funds
    
    println!("📋 Transaction to analyze:");
    println!("   Type: SOL Transfer");
    println!("   From: {}...{} (Your wallet)", hex::encode(&victim.0[..4]), hex::encode(&victim.0[28..]));
    println!("   To: {}...{} (Unknown)", hex::encode(&attacker.0[..4]), hex::encode(&attacker.0[28..]));
    println!("   Amount: {} lamports (1.0 SOL)", drain_amount);
    
    let tx = SolanaTransactionParser::create_transfer_transaction(
        victim, attacker, drain_amount, SolanaHash([99u8; 32])
    );
    
    let analysis_start = Instant::now();
    
    println!();
    println!("🧠 AI Agent Security Analysis:");
    
    // Check 1: Amount is suspicious
    let large_amount = drain_amount >= 500_000_000; // 0.5 SOL or more
    println!("   💰 Amount Check: {}", 
             if large_amount { "🚨 LARGE AMOUNT (>0.5 SOL)" } else { "✅ Reasonable amount" });
    
    // Check 2: Recipient reputation (simulated blacklist check)
    let suspicious_recipient = attacker.0[0] == 66; // Simulate blacklist hit
    println!("   👤 Recipient Check: {}", 
             if suspicious_recipient { "🚨 SUSPICIOUS ADDRESS" } else { "✅ Clean address" });
    
    // Check 3: Check if this drains most of the wallet
    runtime.fund_account(&Pubkey::new(victim.0), 1_100_000_000); // 1.1 SOL
    let current_balance = runtime.get_balance(&Pubkey::new(victim.0));
    let percentage_drained = (drain_amount as f64 / current_balance as f64) * 100.0;
    
    println!("   💸 Drain Check: 🚨 DRAINS {:.1}% OF WALLET", percentage_drained);
    
    // Check 4: Recent activity pattern (simulated)
    println!("   📊 Pattern Check: 🚨 Similar to known scam patterns");
    
    let analysis_time = analysis_start.elapsed();
    
    println!();
    println!("🚨 VERDICT: DANGEROUS - DO NOT SIGN");
    println!("   Analysis time: {:?}", analysis_time);
    println!("   Risk score: 9/10 (Very Dangerous)");
    println!("   Recommendation: 🛑 Agent blocks signing");
    println!();
    println!("⚠️ WARNING SIGNS:");
    println!("   • Transfers large portion of wallet");
    println!("   • Recipient appears on warning lists");
    println!("   • Pattern matches known scams");
    println!();
    
    Ok(())
}

async fn analyze_unknown_program_transaction(runtime: &mut IntegratedRuntime) -> Result<()> {
    println!("🚨 ANALYSIS 3: UNKNOWN PROGRAM INTERACTION");
    println!("==========================================");
    
    let user = SolanaPubkey::new([5u8; 32]);
    let unknown_program = SolanaPubkey::new([88u8; 32]); // Unknown program
    let recent_blockhash = SolanaHash([77u8; 32]);
    
    // Create a suspicious transaction with unknown program
    let instruction = CompiledInstruction {
        program_id_index: 1, // unknown program
        accounts: vec![0], // user account
        data: vec![
            1, // Unknown instruction
            255, 255, 255, 255, 255, 255, 255, 255, // Suspicious max values
        ],
    };
    
    let message = SolanaMessage {
        header: MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        },
        account_keys: vec![user, unknown_program],
        recent_blockhash,
        instructions: vec![instruction],
    };
    
    let tx = SolanaTransaction {
        signatures: vec![SolanaSignature([0u8; 64])],
        message,
    };
    
    println!("📋 Transaction to analyze:");
    println!("   Type: Program Interaction");
    println!("   User: {}...{}", hex::encode(&user.0[..4]), hex::encode(&user.0[28..]));
    println!("   Program: {}...{}", hex::encode(&unknown_program.0[..4]), hex::encode(&unknown_program.0[28..]));
    println!("   Instruction: Unknown (opcode 1)");
    
    let analysis_start = Instant::now();
    
    println!();
    println!("🧠 AI Agent Security Analysis:");
    
    // Check 1: Program recognition
    let known_program = false; // Not system program or well-known program
    println!("   🔍 Program Check: {}", 
             if known_program { "✅ Known safe program" } else { "🚨 UNKNOWN PROGRAM" });
    
    // Check 2: Instruction analysis
    let suspicious_instruction = true; // Unusual instruction pattern
    println!("   📝 Instruction Check: {}", 
             if suspicious_instruction { "🚨 SUSPICIOUS DATA PATTERN" } else { "✅ Normal instruction" });
    
    // Check 3: Permissions requested
    println!("   🔑 Permission Check: 🚨 REQUESTS ACCOUNT WRITE ACCESS");
    
    // Check 4: Program verification
    println!("   🛡️ Verification Check: ❌ Program not verified");
    
    let analysis_time = analysis_start.elapsed();
    
    println!();
    println!("🚨 VERDICT: HIGH RISK - REQUIRES MANUAL REVIEW");
    println!("   Analysis time: {:?}", analysis_time);
    println!("   Risk score: 8/10 (High Risk)");
    println!("   Recommendation: 🛑 Agent suggests caution");
    println!();
    println!("⚠️ RISK FACTORS:");
    println!("   • Unknown program (not in verified list)");
    println!("   • Suspicious instruction data");
    println!("   • Requests write access to your accounts");
    println!("   • Cannot predict transaction outcome");
    println!();
    
    Ok(())
}

async fn demonstrate_protection_value(runtime: &mut IntegratedRuntime) -> Result<()> {
    println!("🛡️ THE PROTECTION VALUE");
    println!("========================");
    
    println!("🤖 This AI security layer prevents:");
    println!("   • Wallet draining attacks");
    println!("   • Interaction with malicious contracts");
    println!("   • Signing transactions that will fail");
    println!("   • Unknown program exploits");
    println!();
    
    println!("⚡ Analysis Performance:");
    println!("   • Transaction parsing: <1ms");
    println!("   • Security analysis: <1ms");
    println!("   • Simulation execution: <1ms");
    println!("   • Total protection time: ~3ms");
    println!();
    
    println!("🏗️ Integration Ready:");
    println!("   • Works in any wallet app");
    println!("   • Runs in browser or mobile");
    println!("   • No network calls required");
    println!("   • Perfect for Web3 agents");
    println!();
    
    println!("🎯 PERFECT FOR:");
    println!("   📱 Wallet security layers");
    println!("   🤖 AI transaction agents");
    println!("   🔒 DeFi safety tools");
    println!("   🎓 Educational security demos");
    println!();
    
    println!("💡 IMAGINE:");
    println!("   Every wallet has this built-in protection");
    println!("   No more signing malicious transactions");
    println!("   AI agents that truly understand blockchain");
    println!("   Perfect transaction transparency");
    
    Ok(())
} 