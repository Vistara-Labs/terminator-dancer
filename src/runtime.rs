use crate::types::*;
use crate::{Result, TerminatorError};
use std::fs;
use tracing::{info, warn, debug};
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logging() {
    INIT.call_once(|| {
        #[cfg(all(feature = "tracing-subscriber", not(feature = "wasm")))]
        {
            tracing_subscriber::fmt::init();
        }
    });
}

#[derive(Debug, Clone)]
pub struct TerminatorRuntime {
    config: RuntimeConfig,
    bank_state: BankState,
}

impl TerminatorRuntime {
    pub async fn new(config_path: &str) -> Result<Self> {
        // Initialize logging for native builds only
        #[cfg(all(feature = "tracing-subscriber", not(feature = "wasm")))]
        tracing_subscriber::fmt::init();
        
        let config = if fs::metadata(config_path).is_ok() {
            let config_str = fs::read_to_string(config_path)
                .map_err(|e| TerminatorError::SerializationError(e.to_string()))?;
            toml::from_str(&config_str)
                .map_err(|e| TerminatorError::SerializationError(e.to_string()))?
        } else {
            warn!("Config file {} not found, using defaults", config_path);
            RuntimeConfig::default()
        };
        
        info!("Initializing Terminator Runtime with config: {:?}", config);
        
        let mut bank_state = BankState::new();
        // Initialize with some lamports from config
        if config.bank.initial_lamports > 0 {
                    let system_account = Pubkey::system_program();
        bank_state.accounts.insert(
            system_account,
            Account::new(config.bank.initial_lamports, vec![], system_account.0),
        );
        }
        
        Ok(Self {
            config,
            bank_state,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Terminator Runtime...");
        info!("Configuration loaded:");
        info!("  Compute budget: {}", self.config.runtime.compute_budget);
        info!("  Max transaction size: {}", self.config.runtime.max_transaction_size);
        info!("  Fuzzing enabled: {}", self.config.runtime.enable_fuzzing);
        info!("  Initial bank lamports: {}", self.config.bank.initial_lamports);
        Ok(())
    }

    pub fn execute_transaction(&mut self, txn: &Transaction) -> Result<TransactionResult> {
        info!("Executing transaction with {} instructions", txn.instructions.len());
        
        let mut execution_context = ExecutionContext::new(self.config.runtime.compute_budget);
        let mut logs = Vec::new();
        
        // Validate transaction size
        let tx_size = bincode::serialized_size(txn)
            .map_err(|e| TerminatorError::SerializationError(e.to_string()))?;
        
        if tx_size as usize > self.config.runtime.max_transaction_size {
            return Err(TerminatorError::TransactionExecutionFailed(
                format!("Transaction too large: {} > {}", tx_size, self.config.runtime.max_transaction_size)
            ));
        }

        // Execute each instruction
        for (i, instruction) in txn.instructions.iter().enumerate() {
            if !execution_context.consume_compute_units(1000) {
                return Err(TerminatorError::TransactionExecutionFailed(
                    "Compute budget exceeded".to_string()
                ));
            }
            
            debug!("Processing instruction {}: {:?}", i, instruction.program_id);
            self.process_instruction(instruction, &mut execution_context)?;
            logs.push(format!("Instruction {} processed successfully", i));
        }
        
        info!("Transaction executed successfully, compute units remaining: {}", 
              execution_context.compute_units_remaining);
        
        Ok(TransactionResult {
            success: true,
            compute_units_consumed: self.config.runtime.compute_budget - execution_context.compute_units_remaining,
            logs: execution_context.log_messages,
            error: None,
        })
    }

    fn process_instruction(&mut self, instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        // Route instruction based on program ID
        match instruction.program_id {
            p if p == Pubkey::system_program() => {
                self.handle_system_instruction(instruction, context)
            }
            p if p == Pubkey::token_program() => {
                self.handle_token_instruction(instruction, context)
            }
            _ => {
                // Generic program handling
                self.handle_generic_instruction(instruction, context)
            }
        }
    }

    fn handle_system_instruction(&mut self, instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        context.log("Processing system program instruction".to_string());
        
        // Handle based on InstructionData
        match &instruction.data {
            InstructionData::Transfer { from, to, lamports } => {
                self.handle_transfer_instruction(*from, *to, *lamports, context)
            }
            InstructionData::CreateAccount { from, to, lamports, space, owner } => {
                self.handle_create_account_instruction(*from, *to, *lamports, *space, *owner, context)
            }
            InstructionData::Assign { account, owner } => {
                self.handle_assign_instruction(*account, *owner, context)
            }
            InstructionData::Generic { data } => {
                // Legacy handling for generic data
                if data.is_empty() {
                    return Ok(());
                }
                
                match data[0] {
                    0 => self.handle_create_account(instruction, context),
                    1 => self.handle_assign(instruction, context),
                    2 => self.handle_transfer(instruction, context),
                    _ => {
                        context.log(format!("Unknown system instruction: {}", data[0]));
                        Ok(())
                    }
                }
            }
        }
    }

    fn handle_transfer_instruction(&mut self, from: [u8; 32], to: [u8; 32], lamports: u64, context: &mut ExecutionContext) -> Result<()> {
        let from_key = Pubkey::new(from);
        let to_key = Pubkey::new(to);
        
        context.log(format!("Transferring {} lamports from {:?} to {:?}", lamports, from_key, to_key));
        
        // Get or create from account - for demo purposes, create with sufficient balance if it doesn't exist
        if !self.bank_state.accounts.contains_key(&from_key) {
            context.log(format!("Creating from account with initial balance for demo"));
            let initial_balance = std::cmp::max(lamports * 2, 10_000_000); // Ensure sufficient balance
            self.bank_state.accounts.insert(from_key, Account::new(initial_balance, vec![], Pubkey::system_program().0));
        }
        
        let from_account = self.bank_state.accounts.get_mut(&from_key)
            .ok_or_else(|| TerminatorError::AccountNotFound(format!("{:?}", from_key)))?;
        
        if from_account.lamports < lamports {
            return Err(TerminatorError::InsufficientFunds);
        }
        
        from_account.lamports -= lamports;
        
        let to_account = self.bank_state.accounts.entry(to_key)
            .or_insert_with(|| Account::new(0, vec![], Pubkey::system_program().0));
        to_account.lamports += lamports;
        
        Ok(())
    }

    fn handle_create_account_instruction(&mut self, _from: [u8; 32], to: [u8; 32], lamports: u64, space: u64, owner: [u8; 32], context: &mut ExecutionContext) -> Result<()> {
        let to_key = Pubkey::new(to);
        
        context.log(format!("Creating account {:?} with {} lamports and {} bytes", to_key, lamports, space));
        
        // Create new account
        let new_account = Account::new(lamports, vec![0u8; space as usize], owner);
        self.bank_state.accounts.insert(to_key, new_account);
        
        Ok(())
    }

    fn handle_assign_instruction(&mut self, account: [u8; 32], owner: [u8; 32], context: &mut ExecutionContext) -> Result<()> {
        let account_key = Pubkey::new(account);
        
        context.log(format!("Assigning account {:?} to owner {:?}", account_key, owner));
        
        if let Some(acc) = self.bank_state.accounts.get_mut(&account_key) {
            acc.owner = owner;
        }
        
        Ok(())
    }

    fn handle_create_account(&mut self, instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        if instruction.accounts.len() < 2 {
            return Err(TerminatorError::TransactionExecutionFailed(
                "CreateAccount requires at least 2 accounts".to_string()
            ));
        }
        
        let _from = &instruction.accounts[0];
        let to = &instruction.accounts[1];
        
        context.log(format!("Creating account: {:?}", to.pubkey));
        
        // Create new account with minimal lamports
        let new_account = Account::new(1_000_000, vec![], Pubkey::system_program().0);
        self.bank_state.accounts.insert(to.pubkey, new_account);
        
        Ok(())
    }

    fn handle_assign(&mut self, _instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        context.log("Handling assign instruction".to_string());
        // Simplified assign implementation
        Ok(())
    }

    fn handle_transfer(&mut self, instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        if instruction.accounts.len() < 2 {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Transfer requires at least 2 accounts".to_string()
            ));
        }
        
        let from_key = instruction.accounts[0].pubkey;
        let to_key = instruction.accounts[1].pubkey;
        
        // Parse lamports from instruction data (simplified)
        let lamports = if let InstructionData::Generic { data } = &instruction.data {
            if data.len() >= 9 {
                u64::from_le_bytes([
                    data[1], data[2], data[3], data[4],
                    data[5], data[6], data[7], data[8],
                ])
            } else {
                1000000 // Default transfer amount
            }
        } else {
            1000000
        };
        
        context.log(format!("Transferring {} lamports from {:?} to {:?}", lamports, from_key, to_key));
        
        // Get or create accounts
        let from_account = self.bank_state.accounts.get_mut(&from_key)
            .ok_or_else(|| TerminatorError::AccountNotFound(format!("{:?}", from_key)))?;
        
        if from_account.lamports < lamports {
            return Err(TerminatorError::InsufficientFunds);
        }
        
        from_account.lamports -= lamports;
        
        let to_account = self.bank_state.accounts.entry(to_key)
            .or_insert_with(|| Account::new(0, vec![], Pubkey::system_program().0));
        to_account.lamports += lamports;
        
        Ok(())
    }

    fn handle_token_instruction(&mut self, _instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        context.log("Processing token program instruction".to_string());
        // Simplified token instruction handling
        Ok(())
    }

    fn handle_generic_instruction(&mut self, _instruction: &Instruction, context: &mut ExecutionContext) -> Result<()> {
        context.log("Processing generic program instruction".to_string());
        // Simplified generic instruction handling
        Ok(())
    }
}

// Add bincode dependency for serialization
impl Transaction {
    pub fn serialized_size(&self) -> usize {
        bincode::serialized_size(self).unwrap_or(0) as usize
    }
}
