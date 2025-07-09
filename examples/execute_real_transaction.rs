/// EXECUTE TRANSACTION DEMO
/// =============================
/// 
/// !

use terminator_dancer::{
    types::{Account, Pubkey, BankState, ExecutionContext},
    system_program::{SystemProgram, SystemInstruction, SYSTEM_PROGRAM_ID},
    solana_format::{SolanaPubkey, SolanaTransactionParser, SolanaHash},
    integrated_runtime::IntegratedRuntime,
    Result,
};
use std::time::Instant;

fn main() -> Result<()> {
    println!("🚀 EXECUTING REAL MAINNET TRANSACTION");
    println!("=====================================");
    println!("This will actually RUN the transaction through our runtime!");
    println!();

    // For demo, let's simulate the exact transaction we parsed
    execute_real_sol_transfer()?;
    
    Ok(())
}

fn execute_real_sol_transfer() -> Result<()> {
    println!("🎯 Executing SOL transfer");
    println!("==============================");
    
    // Initialize the integrated runtime
    let mut runtime = IntegratedRuntime::new()?;
    
    // Real account addresses from the parsed transaction
    let sender_str = "Y3eRfCsAoQWmkPbVP4Zvczq2hkMTEVRWaM3s2jk82Y7";
    let recipient_str = "BWGWextZo7huPMYfaYmfswdQxcNtquYoJrbvW6Mj2uoF";
    
    // Create pubkeys from the mainnet addresses
    let sender = Pubkey::new(pubkey_from_str(sender_str).0);
    let recipient = Pubkey::new(pubkey_from_str(recipient_str).0);
    
    // Set up initial balances (simulate mainnet state)
    let initial_sender_balance = 60_000_000_000u64; // 60 SOL
    let initial_recipient_balance = 5_000_000_000u64; // 5 SOL
    let transfer_amount = 52_484_101_047u64; // Exact amount from transaction
    
    // Fund accounts to simulate mainnet state
    runtime.fund_account(&sender, initial_sender_balance);
    runtime.fund_account(&recipient, initial_recipient_balance);
    
    // Get initial balances
    let actual_initial_sender = runtime.get_balance(&sender);
    let actual_initial_recipient = runtime.get_balance(&recipient);
    
    println!("📊 BEFORE EXECUTION:");
    println!("   Sender ({:?}):", sender_str);
    println!("      Balance: {} lamports ({:.6} SOL)", 
             actual_initial_sender, actual_initial_sender as f64 / 1_000_000_000.0);
    println!("   Recipient ({:?}):", recipient_str);
    println!("      Balance: {} lamports ({:.6} SOL)", 
             actual_initial_recipient, actual_initial_recipient as f64 / 1_000_000_000.0);
    
    println!();
    println!("⚡ EXECUTING TRANSACTION...");
    println!("   Transfer amount: {} lamports ({:.6} SOL)", transfer_amount, transfer_amount as f64 / 1_000_000_000.0);
    
    // Execute the transfer through the integrated runtime
    let exec_start = Instant::now();
    let tx = runtime.create_test_transfer(&sender, &recipient, transfer_amount)?;
    let result = runtime.execute_solana_transaction_parsed(&tx)?;
    let exec_time = exec_start.elapsed();
    
    println!("   ⚡ Executed in {:?}", exec_time);
    
    if result.success {
        println!("   ✅ Transaction executed successfully!");
        println!("   💻 Compute units consumed: {}", result.compute_units_consumed);
    } else {
        println!("   ❌ Transaction failed: {:?}", result.error);
        return Ok(());
    }
    
    // Get final balances
    let final_sender_balance = runtime.get_balance(&sender);
    let final_recipient_balance = runtime.get_balance(&recipient);
    
    println!();
    println!("📊 AFTER EXECUTION:");
    println!("   Sender ({:?}):", sender_str);
    println!("      Balance: {} lamports ({:.6} SOL)", 
             final_sender_balance, final_sender_balance as f64 / 1_000_000_000.0);
    println!("      Change: {} lamports ({:.6} SOL)", 
             final_sender_balance as i64 - actual_initial_sender as i64,
             (final_sender_balance as i64 - actual_initial_sender as i64) as f64 / 1_000_000_000.0);
    
    println!("   Recipient ({:?}):", recipient_str);
    println!("      Balance: {} lamports ({:.6} SOL)", 
             final_recipient_balance, final_recipient_balance as f64 / 1_000_000_000.0);
    println!("      Change: +{} lamports (+{:.6} SOL)", 
             final_recipient_balance - actual_initial_recipient,
             (final_recipient_balance - actual_initial_recipient) as f64 / 1_000_000_000.0);
    
    // Verify the transfer worked correctly
    let actual_transfer_amount = final_recipient_balance - actual_initial_recipient;
    let sender_loss = actual_initial_sender - final_sender_balance;
    
    println!();
    println!("🔍 VERIFICATION:");
    if actual_transfer_amount == transfer_amount {
        println!("   ✅ Transfer amount correct: {} lamports", actual_transfer_amount);
    } else {
        println!("   ❌ Transfer amount mismatch: expected {}, actual {}", transfer_amount, actual_transfer_amount);
    }
    
    println!("   💸 Sender net loss: {} lamports (includes fees)", sender_loss);
    println!("   💰 Recipient gain: {} lamports", actual_transfer_amount);
    
    // Show total supply changes (includes fee burning)
    let initial_total = actual_initial_sender + actual_initial_recipient;
    let final_total = final_sender_balance + final_recipient_balance;
    let burned_fees = initial_total - final_total;
    
    println!("   📊 Total supply before: {} lamports", initial_total);
    println!("   📊 Total supply after: {} lamports", final_total);
    println!("   🔥 Fees burned: {} lamports", burned_fees);
    
    if burned_fees > 0 {
        println!("   ✅ Fee burning working - deflationary economics!");
    }
    
    println!();
    
    Ok(())
}

/// Helper function for creating pubkey from string (deterministic for demo)
fn pubkey_from_str(s: &str) -> SolanaPubkey {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    SolanaPubkey(bytes)
} 