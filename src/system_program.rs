/// Solana System Program Implementation
/// Handles: Transfer, CreateAccount, Assign, Allocate, etc.

use crate::{Result, TerminatorError};
use crate::types::{Account, Pubkey, ExecutionContext};
use borsh::{BorshDeserialize, BorshSerialize};

/// Solana System Program ID (all zeros)
pub const SYSTEM_PROGRAM_ID: [u8; 32] = [0u8; 32];

/// System program instruction types (matches Solana exactly)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum SystemInstruction {
    /// Create a new account
    /// Accounts:
    /// [0] Funding account (signer, writable)
    /// [1] New account (signer, writable)
    CreateAccount {
        lamports: u64,
        space: u64,
        owner: [u8; 32],
    },
    
    /// Assign account to a program
    /// Accounts:
    /// [0] Account to assign (signer, writable)
    Assign {
        owner: [u8; 32],
    },
    
    /// Transfer lamports
    /// Accounts:
    /// [0] Funding account (signer, writable)
    /// [1] Recipient account (writable)
    Transfer {
        lamports: u64,
    },
    
    /// Create account with seed
    /// Accounts:
    /// [0] Funding account (signer, writable)
    /// [1] Created account (writable)
    /// [2] Base account (signer)
    CreateAccountWithSeed {
        base: [u8; 32],
        seed: String,
        lamports: u64,
        space: u64,
        owner: [u8; 32],
    },
    
    /// Allocate space for account data
    /// Accounts:
    /// [0] Account to allocate (signer, writable)
    Allocate {
        space: u64,
    },
    
    /// Allocate space with seed
    AllocateWithSeed {
        base: [u8; 32],
        seed: String,
        space: u64,
        owner: [u8; 32],
    },
    
    /// Assign account with seed
    AssignWithSeed {
        base: [u8; 32],
        seed: String,
        owner: [u8; 32],
    },
    
    /// Transfer with seed
    TransferWithSeed {
        lamports: u64,
        from_seed: String,
        from_owner: [u8; 32],
    },
}

/// System Program processor
pub struct SystemProgram;

impl SystemProgram {
    /// Process a system program instruction
    pub fn process_instruction(
        instruction_data: &[u8],
        account_keys: &[Pubkey],
        account_infos: &mut [&mut Account],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        let instruction = SystemInstruction::try_from_slice(instruction_data)
            .map_err(|_| TerminatorError::SerializationError("Invalid system instruction".to_string()))?;
        
        context.log(format!("Processing system instruction: {:?}", instruction));
        
        match instruction {
            SystemInstruction::CreateAccount { lamports, space, owner } => {
                Self::create_account(account_keys, account_infos, lamports, space, owner, context)
            }
            SystemInstruction::Assign { owner } => {
                Self::assign_account(account_infos, owner, context)
            }
            SystemInstruction::Transfer { lamports } => {
                Self::transfer(account_infos, lamports, context)
            }
            SystemInstruction::CreateAccountWithSeed { base, seed, lamports, space, owner } => {
                Self::create_account_with_seed(account_keys, account_infos, base, &seed, lamports, space, owner, context)
            }
            SystemInstruction::Allocate { space } => {
                Self::allocate(account_infos, space, context)
            }
            SystemInstruction::AllocateWithSeed { base, seed, space, owner } => {
                Self::allocate_with_seed(account_keys, account_infos, base, &seed, space, owner, context)
            }
            SystemInstruction::AssignWithSeed { base, seed, owner } => {
                Self::assign_with_seed(account_keys, account_infos, base, &seed, owner, context)
            }
            SystemInstruction::TransferWithSeed { lamports, from_seed, from_owner } => {
                Self::transfer_with_seed(account_keys, account_infos, lamports, &from_seed, from_owner, context)
            }
        }
    }
    
    /// Create a new account
    fn create_account(
        account_keys: &[Pubkey],
        account_infos: &mut [&mut Account],
        lamports: u64,
        space: u64,
        owner: [u8; 32],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        if account_infos.len() < 2 {
            return Err(TerminatorError::TransactionExecutionFailed(
                "CreateAccount requires 2 accounts".to_string()
            ));
        }
        
        context.log(format!(
            "Creating account {:?} with {} lamports, {} bytes, owner {:?}",
            account_keys.get(1).unwrap_or(&account_keys[0]), lamports, space, owner
        ));
        
        // Check funding account has sufficient balance
        if account_infos[0].lamports < lamports {
            return Err(TerminatorError::InsufficientFunds);
        }
        
        // Use split_at_mut to safely get mutable references
        let (from_accounts, to_accounts) = account_infos.split_at_mut(1);
        let from_account = &mut from_accounts[0];
        let to_account = &mut to_accounts[0];
        
        // Transfer lamports
        from_account.lamports -= lamports;
        to_account.lamports = lamports;
        
        // Set account properties
        to_account.data = vec![0u8; space as usize];
        to_account.owner = owner;
        to_account.executable = false;
        to_account.rent_epoch = 0;
        
        context.consume_compute_units(1000);
        Ok(())
    }
    
    /// Assign account to a program
    fn assign_account(
        account_infos: &mut [&mut Account],
        owner: [u8; 32],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        if account_infos.is_empty() {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Assign requires 1 account".to_string()
            ));
        }
        
        let account = &mut account_infos[0];
        
        context.log(format!("Assigning account to owner {:?}", owner));
        
        // Only system-owned accounts can be assigned
        if account.owner != SYSTEM_PROGRAM_ID {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Only system-owned accounts can be assigned".to_string()
            ));
        }
        
        account.owner = owner;
        
        context.consume_compute_units(500);
        Ok(())
    }
    
    /// Transfer lamports between accounts
    fn transfer(
        account_infos: &mut [&mut Account],
        lamports: u64,
        context: &mut ExecutionContext,
    ) -> Result<()> {
        if account_infos.len() < 2 {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Transfer requires 2 accounts".to_string()
            ));
        }
        
        context.log(format!("Transferring {} lamports", lamports));
        
        // Check sufficient funds
        if account_infos[0].lamports < lamports {
            return Err(TerminatorError::InsufficientFunds);
        }
        
        // Use split_at_mut to safely get mutable references
        let (from_accounts, to_accounts) = account_infos.split_at_mut(1);
        let from_account = &mut from_accounts[0];
        let to_account = &mut to_accounts[0];
        
        // Transfer
        from_account.lamports -= lamports;
        to_account.lamports += lamports;
        
        context.consume_compute_units(200);
        Ok(())
    }
    
    /// Create account with seed (simplified implementation)
    fn create_account_with_seed(
        _account_keys: &[Pubkey],
        account_infos: &mut [&mut Account],
        _base: [u8; 32],
        _seed: &str,
        lamports: u64,
        space: u64,
        owner: [u8; 32],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        // For now, treat like regular create account
        Self::create_account(&[], account_infos, lamports, space, owner, context)
    }
    
    /// Allocate space for account data
    fn allocate(
        account_infos: &mut [&mut Account],
        space: u64,
        context: &mut ExecutionContext,
    ) -> Result<()> {
        if account_infos.is_empty() {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Allocate requires 1 account".to_string()
            ));
        }
        
        let account = &mut account_infos[0];
        
        context.log(format!("Allocating {} bytes", space));
        
        // Only system-owned accounts can be allocated
        if account.owner != SYSTEM_PROGRAM_ID {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Only system-owned accounts can be allocated".to_string()
            ));
        }
        
        account.data = vec![0u8; space as usize];
        
        context.consume_compute_units(space / 100); // Proportional to space
        Ok(())
    }
    
    /// Placeholder implementations for seed-based operations
    fn allocate_with_seed(
        _account_keys: &[Pubkey],
        account_infos: &mut [&mut Account],
        _base: [u8; 32],
        _seed: &str,
        space: u64,
        _owner: [u8; 32],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        Self::allocate(account_infos, space, context)
    }
    
    fn assign_with_seed(
        _account_keys: &[Pubkey],
        account_infos: &mut [&mut Account],
        _base: [u8; 32],
        _seed: &str,
        owner: [u8; 32],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        Self::assign_account(account_infos, owner, context)
    }
    
    fn transfer_with_seed(
        _account_keys: &[Pubkey],
        account_infos: &mut [&mut Account],
        lamports: u64,
        _from_seed: &str,
        _from_owner: [u8; 32],
        context: &mut ExecutionContext,
    ) -> Result<()> {
        Self::transfer(account_infos, lamports, context)
    }
}

/// Helper functions for creating system instructions
impl SystemInstruction {
    /// Create a transfer instruction
    pub fn transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> (Self, Vec<Pubkey>) {
        let instruction = SystemInstruction::Transfer { lamports };
        let accounts = vec![*from, *to];
        (instruction, accounts)
    }
    
    /// Create an account creation instruction
    pub fn create_account(
        from: &Pubkey,
        to: &Pubkey,
        lamports: u64,
        space: u64,
        owner: &[u8; 32],
    ) -> (Self, Vec<Pubkey>) {
        let instruction = SystemInstruction::CreateAccount {
            lamports,
            space,
            owner: *owner,
        };
        let accounts = vec![*from, *to];
        (instruction, accounts)
    }
    
    /// Create an assign instruction
    pub fn assign(account: &Pubkey, owner: &[u8; 32]) -> (Self, Vec<Pubkey>) {
        let instruction = SystemInstruction::Assign { owner: *owner };
        let accounts = vec![*account];
        (instruction, accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_instruction_serialization() {
        let instruction = SystemInstruction::Transfer { lamports: 1000000 };
        let serialized = borsh::to_vec(&instruction).unwrap();
        let deserialized: SystemInstruction = borsh::from_slice(&serialized).unwrap();
        
        match deserialized {
            SystemInstruction::Transfer { lamports } => assert_eq!(lamports, 1000000),
            _ => panic!("Wrong instruction type"),
        }
    }
    
    #[test]
    fn test_create_transfer_instruction() {
        let from = Pubkey::new([1u8; 32]);
        let to = Pubkey::new([2u8; 32]);
        let (instruction, accounts) = SystemInstruction::transfer(&from, &to, 5000);
        
        match instruction {
            SystemInstruction::Transfer { lamports } => assert_eq!(lamports, 5000),
            _ => panic!("Wrong instruction type"),
        }
        
        assert_eq!(accounts, vec![from, to]);
    }
} 