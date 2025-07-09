/// WASM-compatible Terminator-Dancer Runtime
/// Runs entirely in the browser with real-time metrics and interactive features

use crate::{Result, TerminatorError};
use crate::types::{Account, Pubkey, ExecutionContext, TransactionResult};
use crate::system_program::{SystemProgram, SYSTEM_PROGRAM_ID};
use crate::solana_format::{SolanaTransaction, SolanaTransactionParser, SolanaPubkey, SolanaHash};
use crate::crypto::SolanaCrypto;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, Performance};

/// WASM-compatible runtime for browser execution
#[wasm_bindgen]
pub struct WasmRuntime {
    accounts: HashMap<Pubkey, Account>,
    compute_budget: u64,
    transaction_count: u64,
    total_execution_time: f64,
    performance: Performance,
}

/// Performance metrics for real-time display
#[wasm_bindgen]
#[derive(Clone)]
pub struct PerformanceMetrics {
    tps: f64,
    total_transactions: u64,
    total_time_ms: f64,
    avg_execution_time_us: f64,
    successful_transactions: u64,
    failed_transactions: u64,
}

#[wasm_bindgen]
impl WasmRuntime {
    /// Create new WASM runtime instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> std::result::Result<WasmRuntime, JsValue> {
        console::log_1(&"🚀 Initializing Terminator-Dancer WASM Runtime".into());
        
        // Get browser performance API
        let window = web_sys::window().ok_or("No window object")?;
        let performance = window.performance().ok_or("No performance API")?;
        
        let mut runtime = WasmRuntime {
            accounts: HashMap::new(),
            compute_budget: 1_400_000,
            transaction_count: 0,
            total_execution_time: 0.0,
            performance,
        };
        
        // Initialize default accounts
        runtime.initialize_default_accounts()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        console::log_1(&"✅ WASM Runtime initialized successfully".into());
        Ok(runtime)
    }
    
    /// Execute a single Solana transaction and return metrics
    #[wasm_bindgen]
    pub fn execute_transaction(&mut self, from_hex: &str, to_hex: &str, amount: u64) -> std::result::Result<PerformanceMetrics, JsValue> {
        let start_time = self.performance.now();
        
        // Parse addresses
        let from_bytes = hex::decode(from_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid from address: {}", e)))?;
        let to_bytes = hex::decode(to_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid to address: {}", e)))?;
        
        if from_bytes.len() != 32 || to_bytes.len() != 32 {
            return Err(JsValue::from_str("Addresses must be 32 bytes"));
        }
        
        let mut from_array = [0u8; 32];
        let mut to_array = [0u8; 32];
        from_array.copy_from_slice(&from_bytes);
        to_array.copy_from_slice(&to_bytes);
        
        let from_pubkey = Pubkey::new(from_array);
        let to_pubkey = Pubkey::new(to_array);
        
        // Create transaction
        let tx = self.create_transfer_transaction(&from_pubkey, &to_pubkey, amount)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Execute transaction
        let result = self.execute_solana_transaction_internal(&tx)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let end_time = self.performance.now();
        let execution_time = end_time - start_time;
        
        // Update metrics
        self.transaction_count += 1;
        self.total_execution_time += execution_time;
        
        if result.success {
            console::log_1(&format!("✅ Transaction executed: {} lamports transferred", amount).into());
        } else {
            console::log_1(&format!("❌ Transaction failed: {:?}", result.error).into());
        }
        
        Ok(self.get_performance_metrics())
    }
    
    /// Execute multiple transactions for performance testing
    #[wasm_bindgen]
    pub fn execute_batch_transactions(&mut self, count: u32, amount: u64) -> std::result::Result<PerformanceMetrics, JsValue> {
        console::log_1(&format!("🚀 Executing {} transactions for performance test", count).into());
        
        let batch_start = self.performance.now();
        let mut successful = 0;
        let mut failed = 0;
        
        for i in 0..count {
            let from_array = [1u8; 32];
            let mut to_array = [2u8; 32];
            to_array[31] = (i % 256) as u8; // Vary recipient
            
            let from_pubkey = Pubkey::new(from_array);
            let to_pubkey = Pubkey::new(to_array);
            
            // Ensure recipient account exists
            if !self.accounts.contains_key(&to_pubkey) {
                self.accounts.insert(to_pubkey, Account::new(0, vec![], SYSTEM_PROGRAM_ID));
            }
            
            match self.create_and_execute_transfer(&from_pubkey, &to_pubkey, amount) {
                Ok(_) => successful += 1,
                Err(_) => failed += 1,
            }
        }
        
        let batch_end = self.performance.now();
        let batch_time = batch_end - batch_start;
        
        console::log_1(&format!("📊 Batch completed: {} successful, {} failed in {:.2}ms", 
                               successful, failed, batch_time).into());
        
        Ok(self.get_performance_metrics())
    }
    
    /// Get current performance metrics
    #[wasm_bindgen]
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.get_performance_metrics()
    }
    
    /// Get account balance by hex address
    #[wasm_bindgen]
    pub fn get_balance(&self, address_hex: &str) -> std::result::Result<u64, JsValue> {
        let address_bytes = hex::decode(address_hex)
            .map_err(|e| JsValue::from_str(&format!("Invalid address: {}", e)))?;
        
        if address_bytes.len() != 32 {
            return Err(JsValue::from_str("Address must be 32 bytes"));
        }
        
        let mut address_array = [0u8; 32];
        address_array.copy_from_slice(&address_bytes);
        let pubkey = Pubkey::new(address_array);
        
        Ok(self.accounts.get(&pubkey).map(|acc| acc.lamports).unwrap_or(0))
    }
    
    /// Reset runtime state for fresh demo
    #[wasm_bindgen]
    pub fn reset(&mut self) -> std::result::Result<(), JsValue> {
        console::log_1(&"🔄 Resetting runtime state".into());
        
        self.accounts.clear();
        self.transaction_count = 0;
        self.total_execution_time = 0.0;
        
        self.initialize_default_accounts()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        console::log_1(&"✅ Runtime reset complete".into());
        Ok(())
    }
    
    /// Test cryptographic operations for demo
    #[wasm_bindgen]
    pub fn test_crypto_performance(&self, iterations: u32) -> std::result::Result<f64, JsValue> {
        console::log_1(&format!("🔐 Testing crypto performance with {} iterations", iterations).into());
        
        let start_time = self.performance.now();
        
        let message = b"Terminator-Dancer WASM crypto test message";
        
        for _i in 0..iterations {
            let _hash = SolanaCrypto::sha256_hash(message);
        }
        
        let end_time = self.performance.now();
        let total_time = end_time - start_time;
        let ops_per_second = (iterations as f64) / (total_time / 1000.0);
        
        console::log_1(&format!("✅ Crypto test: {:.0} ops/second", ops_per_second).into());
        
        Ok(ops_per_second)
    }
}

// Internal implementation
impl WasmRuntime {
    fn initialize_default_accounts(&mut self) -> Result<()> {
        // System program account
        let system_program_key = Pubkey::new(SYSTEM_PROGRAM_ID);
        let system_account = Account::new_executable(1, vec![], SYSTEM_PROGRAM_ID);
        self.accounts.insert(system_program_key, system_account);
        
        // Create funded test account
        let test_account_key = Pubkey::new([1u8; 32]);
        let test_account = Account::new(10_000_000_000, vec![], SYSTEM_PROGRAM_ID); // 10 SOL
        self.accounts.insert(test_account_key, test_account);
        
        console::log_1(&"✅ Default accounts initialized".into());
        Ok(())
    }
    
    fn create_transfer_transaction(&self, from: &Pubkey, to: &Pubkey, lamports: u64) -> Result<SolanaTransaction> {
        let from_solana = SolanaPubkey::new(from.0);
        let to_solana = SolanaPubkey::new(to.0);
        let blockhash = SolanaHash([0u8; 32]); // Mock blockhash for demo
        
        Ok(SolanaTransactionParser::create_transfer_transaction(
            from_solana,
            to_solana,
            lamports,
            blockhash,
        ))
    }
    
    fn create_and_execute_transfer(&mut self, from: &Pubkey, to: &Pubkey, amount: u64) -> Result<TransactionResult> {
        let tx = self.create_transfer_transaction(from, to, amount)?;
        self.execute_solana_transaction_internal(&tx)
    }
    
    fn execute_solana_transaction_internal(&mut self, solana_tx: &SolanaTransaction) -> Result<TransactionResult> {
        let mut context = ExecutionContext::new(self.compute_budget);
        
        // Process each instruction
        for instruction in &solana_tx.message.instructions {
            // Check compute budget
            if !context.consume_compute_units(1000) {
                return Err(TerminatorError::TransactionExecutionFailed(
                    "Compute budget exceeded".to_string()
                ));
            }
            
            // Get program ID
            if instruction.program_id_index >= solana_tx.message.account_keys.len() as u8 {
                return Err(TerminatorError::TransactionExecutionFailed(
                    "Invalid program_id_index".to_string()
                ));
            }
            
            let program_id = solana_tx.message.account_keys[instruction.program_id_index as usize].0;
            
            // Execute instruction
            self.execute_instruction(
                &program_id,
                &instruction.data,
                &solana_tx.message.account_keys,
                &instruction.accounts,
                &mut context,
            )?;
        }
        
        Ok(TransactionResult {
            success: true,
            compute_units_consumed: self.compute_budget - context.compute_units_remaining,
            logs: context.log_messages,
            error: None,
        })
    }
    
    fn execute_instruction(
        &mut self,
        program_id: &[u8; 32],
        instruction_data: &[u8],
        account_keys: &[SolanaPubkey],
        account_indices: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        // Convert account keys
        let pubkeys: Vec<Pubkey> = account_keys.iter()
            .map(|pk| Pubkey::new(pk.0))
            .collect();
        
        // Ensure accounts exist
        for &index in account_indices {
            if index >= pubkeys.len() as u8 {
                return Err(TerminatorError::TransactionExecutionFailed(
                    "Invalid account index".to_string()
                ));
            }
            
            let pubkey = &pubkeys[index as usize];
            
            if !self.accounts.contains_key(pubkey) {
                let new_account = Account::new(0, vec![], SYSTEM_PROGRAM_ID);
                self.accounts.insert(*pubkey, new_account);
            }
        }
        
        // Execute based on program
        match *program_id {
            SYSTEM_PROGRAM_ID => {
                // Get account references for system program
                let mut account_infos: Vec<Account> = account_indices.iter()
                    .map(|&index| {
                        let pubkey = &pubkeys[index as usize];
                        self.accounts.get(pubkey).cloned().unwrap()
                    })
                    .collect();
                
                let mut account_refs: Vec<&mut Account> = account_infos.iter_mut().collect();
                
                // Execute system program instruction
                SystemProgram::process_instruction(
                    instruction_data,
                    &pubkeys,
                    &mut account_refs,
                    context,
                )?;
                
                // Update accounts back to storage
                for (i, &index) in account_indices.iter().enumerate() {
                    let pubkey = &pubkeys[index as usize];
                    self.accounts.insert(*pubkey, account_infos[i].clone());
                }
            }
            _ => {
                // WASM limitation: Real BPF VM not available in browser (native dependencies)
                context.log(format!("🌐 WASM BPF simulation: {:?}", program_id));
                context.log("⚠️ Real BPF execution available in native runtime only".to_string());
                context.consume_compute_units(1000);
            }
        }
        
        Ok(())
    }
    
    fn get_performance_metrics(&self) -> PerformanceMetrics {
        let tps = if self.total_execution_time > 0.0 {
            (self.transaction_count as f64) / (self.total_execution_time / 1000.0)
        } else {
            0.0
        };
        
        let avg_execution_time = if self.transaction_count > 0 {
            (self.total_execution_time * 1000.0) / (self.transaction_count as f64) // Convert to microseconds
        } else {
            0.0
        };
        
        PerformanceMetrics {
            tps,
            total_transactions: self.transaction_count,
            total_time_ms: self.total_execution_time,
            avg_execution_time_us: avg_execution_time,
            successful_transactions: self.transaction_count, // Simplified for demo
            failed_transactions: 0,
        }
    }
}

#[wasm_bindgen]
impl PerformanceMetrics {
    #[wasm_bindgen(getter)]
    pub fn tps(&self) -> f64 { self.tps }
    
    #[wasm_bindgen(getter)]
    pub fn total_transactions(&self) -> u64 { self.total_transactions }
    
    #[wasm_bindgen(getter)]
    pub fn total_time_ms(&self) -> f64 { self.total_time_ms }
    
    #[wasm_bindgen(getter)]
    pub fn avg_execution_time_us(&self) -> f64 { self.avg_execution_time_us }
    
    #[wasm_bindgen(getter)]
    pub fn successful_transactions(&self) -> u64 { self.successful_transactions }
    
    #[wasm_bindgen(getter)]
    pub fn failed_transactions(&self) -> u64 { self.failed_transactions }
}

/// Initialize WASM runtime - called from JavaScript
#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"🤖 Terminator-Dancer WASM Module Loaded!".into());
    console::log_1(&"Ready to execute blockchain transactions in your browser!".into());
} 