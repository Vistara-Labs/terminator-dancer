pub mod conformance;
pub mod firedancer_integration;
pub mod firedancer_bindings;
pub mod integrated_runtime;
pub mod system_program;
pub mod runtime;
pub mod solana_format;
pub mod types;
pub mod crypto;
pub mod fuzzing;
pub mod real_bpf_vm; // Real Solana BPF VM integration

// WASM-specific modules
#[cfg(feature = "wasm")]
pub mod wasm_runtime;

// Export public API
pub use types::*;
pub use crypto::*;
pub use runtime::*;
pub use integrated_runtime::IntegratedRuntime;
pub use conformance::ConformanceHarness;
pub use firedancer_integration::{FiredancerCrypto, FiredancerValidator, FiredancerConformanceTest};
pub use solana_format::{SolanaTransaction, SolanaTransactionParser, SolanaPubkey, SolanaHash};
pub use system_program::{SystemProgram, SystemInstruction, SYSTEM_PROGRAM_ID};
pub use real_bpf_vm::RealBpfVm;

// WASM exports
#[cfg(feature = "wasm")]
pub use wasm_runtime::WasmRuntime;

#[cfg(feature = "firedancer")]
pub use firedancer_bindings::{FiredancerCrypto as FiredancerCryptoNative, FiredancerVM, FiredancerAccountManager};

#[derive(Debug, thiserror::Error)]
pub enum TerminatorError {
    #[error("Transaction execution failed: {0}")]
    TransactionExecutionFailed(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Program error: {0}")]
    ProgramError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Conformance test failed: {0}")]
    ConformanceTestFailed(String),
    
    #[error("BPF VM error: {0}")]
    BpfVmError(String),
    
    #[error("Firedancer integration error: {0}")]
    FiredancerError(String),
    
    #[error("WASM error: {0}")]
    WasmError(String),
}

pub type Result<T> = std::result::Result<T, TerminatorError>;

/// Runtime configuration and feature detection
pub struct RuntimeCapabilities {
    pub firedancer_available: bool,
    pub crypto_acceleration: bool,
    pub bpf_vm: bool,
    pub account_management: bool,
    pub wasm_mode: bool,
}

impl RuntimeCapabilities {
    pub fn detect() -> Self {
        RuntimeCapabilities {
            firedancer_available: cfg!(feature = "firedancer"),
            crypto_acceleration: true, // Always available with pure Rust crypto
            bpf_vm: cfg!(feature = "firedancer"),
            account_management: true,
            wasm_mode: cfg!(feature = "wasm"),
        }
    }
    
    pub fn print_summary(&self) {
        #[cfg(feature = "wasm")]
        {
            web_sys::console::log_1(&"🤖 Terminator-Dancer Runtime Capabilities (WASM Mode):".into());
            web_sys::console::log_1(&format!("   🌐 WASM Runtime:         {}", if self.wasm_mode { "✅ ACTIVE" } else { "❌ DISABLED" }).into());
            web_sys::console::log_1(&format!("   🔐 Crypto Acceleration:  {}", if self.crypto_acceleration { "✅ ENABLED" } else { "❌ DISABLED" }).into());
            web_sys::console::log_1(&format!("   💾 Account Management:   {}", if self.account_management { "✅ ENABLED" } else { "❌ DISABLED" }).into());
            web_sys::console::log_1(&format!("   🧠 BPF Virtual Machine:  {}", if self.bpf_vm { "✅ AVAILABLE" } else { "⚠️  Mock Mode" }).into());
        }
        
        #[cfg(not(feature = "wasm"))]
        {
            println!("🤖 Terminator-Dancer Runtime Capabilities:");
            println!("   🔥 Firedancer Integration:  {}", if self.firedancer_available { "✅ AVAILABLE" } else { "⚠️  Fallback Mode" });
            println!("   🔐 Crypto Acceleration:     {}", if self.crypto_acceleration { "✅ ENABLED" } else { "❌ DISABLED" });
            println!("   🧠 BPF Virtual Machine:     {}", if self.bpf_vm { "✅ AVAILABLE" } else { "⚠️  Mock Mode" });
            println!("   💾 Account Management:      {}", if self.account_management { "✅ ENABLED" } else { "❌ DISABLED" });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::*;
    use crate::types::*;

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_runtime_initialization() {
        let runtime = TerminatorRuntime::new("nonexistent_config.toml").await;
        assert!(runtime.is_ok());
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_transaction_execution() {
        let mut runtime = TerminatorRuntime::new("nonexistent_config.toml").await.unwrap();
        
        let program_id = Pubkey::new_unique();
        let account = Pubkey::new_unique();
        
        let instruction = Instruction {
            program_id,
            accounts: vec![AccountMeta {
                pubkey: account,
                is_signer: true,
                is_writable: true,
            }],
            data: InstructionData::Generic { data: vec![1, 2, 3, 4] },
        };

        let transaction = Transaction {
            instructions: vec![instruction],
            signatures: vec![[0u8; 64]],
            payer: account.0,
            recent_blockhash: [1u8; 32],
        };

        let result = runtime.execute_transaction(&transaction);
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_conformance_harness() {
        let mut harness = ConformanceHarness::new();
        
        harness.run_test("test_pass", || Ok(()));
        harness.run_test("test_fail", || Err(TerminatorError::TransactionExecutionFailed("test".to_string())));
        
        assert_eq!(harness.passed, 1);
        assert_eq!(harness.failed, 1);
    }

    #[test]
    fn test_fuzzer() {
        use crate::fuzzing::RuntimeFuzzer;
        let fuzzer = RuntimeFuzzer::new(5);
        assert_eq!(fuzzer.iterations, 5);
        
        let transaction = fuzzer.generate_random_transaction();
        assert_eq!(transaction.instructions.len(), 1);
    }
    
    #[test]
    fn test_capabilities_detection() {
        let caps = RuntimeCapabilities::detect();
        assert!(caps.crypto_acceleration);
        assert!(caps.account_management);
    }
}
