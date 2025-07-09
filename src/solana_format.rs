use crate::{Result, TerminatorError};
use serde::{Deserialize, Serialize};
// use serde_with::{serde_as, Bytes}; // Unused imports

/// Real Solana transaction format compatible with Solana's wire format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaTransaction {
    pub signatures: Vec<SolanaSignature>,
    pub message: SolanaMessage,
}

/// Versioned transaction that supports both legacy and v0 formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTransaction {
    pub signatures: Vec<SolanaSignature>,
    pub message: VersionedMessage,
}

/// Message that can be either legacy or v0 format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionedMessage {
    Legacy(SolanaMessage),
    V0(V0Message),
}

/// V0 message format with address lookup table support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V0Message {
    pub header: MessageHeader,
    pub account_keys: Vec<SolanaPubkey>,
    pub recent_blockhash: SolanaHash,
    pub instructions: Vec<CompiledInstruction>,
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

/// Address lookup table reference in v0 transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAddressTableLookup {
    pub account_key: SolanaPubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMessage {
    pub header: MessageHeader,
    pub account_keys: Vec<SolanaPubkey>,
    pub recent_blockhash: SolanaHash,
    pub instructions: Vec<CompiledInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SolanaPubkey(#[serde(with = "serde_bytes")] pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaSignature(#[serde(with = "serde_bytes")] pub [u8; 64]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaHash(#[serde(with = "serde_bytes")] pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>, // Account indices
    pub data: Vec<u8>,
}

impl SolanaPubkey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn new_unique() -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        Self(hash)
    }

    /// System program ID
    pub fn system_program() -> Self {
        Self([0u8; 32])
    }

    /// SPL Token program ID  
    pub fn token_program() -> Self {
        Self([
            6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172,
            28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
        ])
    }

    /// Parse from base58 string (like Solana CLI)
    pub fn from_str(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| TerminatorError::SerializationError("Invalid base58".to_string()))?;
        
        if bytes.len() != 32 {
            return Err(TerminatorError::SerializationError("Invalid pubkey length".to_string()));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }

    /// Convert to base58 string (like Solana CLI)
    pub fn to_string(&self) -> String {
        bs58::encode(&self.0).into_string()
    }
}

impl std::fmt::Display for SolanaPubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Real Solana transaction parser and builder with v0 support
pub struct SolanaTransactionParser;

impl SolanaTransactionParser {
    /// Parse a transaction from Solana's wire format (supports both legacy and v0)
    pub fn parse_transaction(data: &[u8]) -> Result<SolanaTransaction> {
        // First try direct bincode deserialization
        if let Ok(tx) = bincode::deserialize::<SolanaTransaction>(data) {
            return Ok(tx);
        }

        // If that fails, try manual parsing for complex cases
        Self::parse_transaction_manual(data)
    }

    /// Manual transaction parsing for cases where bincode fails
    fn parse_transaction_manual(data: &[u8]) -> Result<SolanaTransaction> {
        if data.is_empty() {
            return Err(TerminatorError::SerializationError("Empty transaction data".to_string()));
        }

        let mut offset = 0;
        
        // Parse signature count
        let num_signatures = data[0] as usize;
        offset += 1;

        // Parse signatures
        let mut signatures = Vec::new();
        for _ in 0..num_signatures {
            if offset + 64 > data.len() {
                return Err(TerminatorError::SerializationError("Incomplete signature data".to_string()));
            }
            let mut sig_bytes = [0u8; 64];
            sig_bytes.copy_from_slice(&data[offset..offset + 64]);
            signatures.push(SolanaSignature(sig_bytes));
            offset += 64;
        }

        // Check for compact encoding before message
        // Solana RPC often includes message length as compact-u16
        if offset < data.len() && data[offset] >= 0x80 {
            // Skip compact-encoded length field
            if data[offset] < 0x80 {
                offset += 1; // Single byte
            } else if offset + 1 < data.len() {
                offset += 2; // Two byte compact encoding
            } else {
                return Err(TerminatorError::SerializationError("Incomplete compact encoding".to_string()));
            }
        }

        // Parse message
        let message = Self::parse_message_manual(&data[offset..])?;

        Ok(SolanaTransaction {
            signatures,
            message,
        })
    }

    /// Manual message parsing
    fn parse_message_manual(data: &[u8]) -> Result<SolanaMessage> {
        let mut offset = 0;
        
        // First try to deserialize the message portion directly
        if let Ok(message) = bincode::deserialize::<SolanaMessage>(data) {
            return Ok(message);
        }

        // If bincode fails, try manual parsing
        if offset + 3 > data.len() {
            return Err(TerminatorError::SerializationError("Incomplete message header".to_string()));
        }

        // Parse message header
        let header = MessageHeader {
            num_required_signatures: data[offset],
            num_readonly_signed_accounts: data[offset + 1],
            num_readonly_unsigned_accounts: data[offset + 2],
        };
        offset += 3;

        // Validate header makes sense
        if header.num_required_signatures > 16 || 
           header.num_readonly_signed_accounts > 16 || 
           header.num_readonly_unsigned_accounts > 16 {
            return Err(TerminatorError::SerializationError(
                format!("Invalid header values: req_sigs={}, ro_signed={}, ro_unsigned={}", 
                    header.num_required_signatures, 
                    header.num_readonly_signed_accounts, 
                    header.num_readonly_unsigned_accounts)
            ));
        }

        // Parse account keys count
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing account keys count".to_string()));
        }
        let num_account_keys = data[offset] as usize;
        offset += 1;

        // Validate account keys count
        if num_account_keys > 64 {
            return Err(TerminatorError::SerializationError(
                format!("Too many account keys: {}", num_account_keys)
            ));
        }

        // Parse account keys
        let mut account_keys = Vec::new();
        for _ in 0..num_account_keys {
            if offset + 32 > data.len() {
                return Err(TerminatorError::SerializationError("Incomplete account key".to_string()));
            }
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&data[offset..offset + 32]);
            account_keys.push(SolanaPubkey(key_bytes));
            offset += 32;
        }

        // Parse recent blockhash
        if offset + 32 > data.len() {
            return Err(TerminatorError::SerializationError("Incomplete recent blockhash".to_string()));
        }
        let mut blockhash_bytes = [0u8; 32];
        blockhash_bytes.copy_from_slice(&data[offset..offset + 32]);
        let recent_blockhash = SolanaHash(blockhash_bytes);
        offset += 32;

        // Parse instructions count
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing instructions count".to_string()));
        }
        let num_instructions = data[offset] as usize;
        offset += 1;

        // Validate instructions count
        if num_instructions > 64 {
            return Err(TerminatorError::SerializationError(
                format!("Too many instructions: {}", num_instructions)
            ));
        }

        // Parse instructions
        let mut instructions = Vec::new();
        for i in 0..num_instructions {
            // Parse program_id_index
            if offset >= data.len() {
                return Err(TerminatorError::SerializationError(
                    format!("Missing program_id_index for instruction {}", i)
                ));
            }
            let program_id_index = data[offset];
            offset += 1;

            // Validate program_id_index
            if program_id_index >= num_account_keys as u8 {
                return Err(TerminatorError::SerializationError(
                    format!("Invalid program_id_index {} for instruction {}", program_id_index, i)
                ));
            }

            // Parse accounts count
            if offset >= data.len() {
                return Err(TerminatorError::SerializationError(
                    format!("Missing accounts count for instruction {}", i)
                ));
            }
            let accounts_count = data[offset] as usize;
            offset += 1;

            // Validate accounts count
            if accounts_count > 64 {
                return Err(TerminatorError::SerializationError(
                    format!("Too many accounts {} for instruction {}", accounts_count, i)
                ));
            }

            // Parse account indices
            if offset + accounts_count > data.len() {
                return Err(TerminatorError::SerializationError(
                    format!("Incomplete accounts for instruction {}", i)
                ));
            }
            let accounts = data[offset..offset + accounts_count].to_vec();
            offset += accounts_count;

            // Validate account indices
            for &account_index in &accounts {
                if account_index >= num_account_keys as u8 {
                    return Err(TerminatorError::SerializationError(
                        format!("Invalid account index {} for instruction {}", account_index, i)
                    ));
                }
            }

            // Parse instruction data length
            if offset >= data.len() {
                return Err(TerminatorError::SerializationError(
                    format!("Missing data length for instruction {}", i)
                ));
            }
            let data_length = data[offset] as usize;
            offset += 1;

            // Validate data length
            if data_length > 1232 { // Solana instruction data limit
                return Err(TerminatorError::SerializationError(
                    format!("Instruction data too large: {} bytes for instruction {}", data_length, i)
                ));
            }

            // Parse instruction data
            if offset + data_length > data.len() {
                return Err(TerminatorError::SerializationError(
                    format!("Incomplete instruction data for instruction {}", i)
                ));
            }
            let instruction_data = data[offset..offset + data_length].to_vec();
            offset += data_length;

            instructions.push(CompiledInstruction {
                program_id_index,
                accounts,
                data: instruction_data,
            });
        }

        Ok(SolanaMessage {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        })
    }

    /// Parse versioned transaction (v0 or legacy)
    pub fn parse_versioned_transaction(data: &[u8]) -> Result<VersionedTransaction> {
        if data.is_empty() {
            return Err(TerminatorError::SerializationError("Empty transaction data".to_string()));
        }

        let first_byte = data[0];
        
        // Check if this is a v0 transaction (first byte has MSB set)
        if first_byte & 0x80 != 0 {
            Self::parse_v0_transaction(data)
        } else {
            Self::parse_legacy_versioned_transaction(data)
        }
    }

    /// Parse v0 transaction format
    fn parse_v0_transaction(data: &[u8]) -> Result<VersionedTransaction> {
        let mut offset = 0;
        
        // Parse signature count (first byte with MSB cleared)
        let num_signatures = (data[0] & 0x7F) as usize;
        offset += 1;
        
        // Parse signatures
        let mut signatures = Vec::new();
        for _ in 0..num_signatures {
            if offset + 64 > data.len() {
                return Err(TerminatorError::SerializationError("Invalid signature data".to_string()));
            }
            let mut sig_bytes = [0u8; 64];
            sig_bytes.copy_from_slice(&data[offset..offset + 64]);
            signatures.push(SolanaSignature(sig_bytes));
            offset += 64;
        }

        // Parse v0 message
        let message_data = &data[offset..];
        let v0_message = Self::parse_v0_message(message_data)?;

        Ok(VersionedTransaction {
            signatures,
            message: VersionedMessage::V0(v0_message),
        })
    }

    /// Parse v0 message format
    fn parse_v0_message(data: &[u8]) -> Result<V0Message> {
        let mut offset = 0;
        
        // Parse header
        if offset + 3 > data.len() {
            return Err(TerminatorError::SerializationError("Invalid header data".to_string()));
        }
        let header = MessageHeader {
            num_required_signatures: data[offset],
            num_readonly_signed_accounts: data[offset + 1],
            num_readonly_unsigned_accounts: data[offset + 2],
        };
        offset += 3;

        // Parse account keys length and keys
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing account keys length".to_string()));
        }
        let num_account_keys = data[offset] as usize;
        offset += 1;

        let mut account_keys = Vec::new();
        for _ in 0..num_account_keys {
            if offset + 32 > data.len() {
                return Err(TerminatorError::SerializationError("Invalid account key data".to_string()));
            }
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&data[offset..offset + 32]);
            account_keys.push(SolanaPubkey(key_bytes));
            offset += 32;
        }

        // Parse recent blockhash
        if offset + 32 > data.len() {
            return Err(TerminatorError::SerializationError("Invalid blockhash data".to_string()));
        }
        let mut blockhash_bytes = [0u8; 32];
        blockhash_bytes.copy_from_slice(&data[offset..offset + 32]);
        let recent_blockhash = SolanaHash(blockhash_bytes);
        offset += 32;

        // Parse instructions
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing instructions length".to_string()));
        }
        let num_instructions = data[offset] as usize;
        offset += 1;

        let mut instructions = Vec::new();
        for _ in 0..num_instructions {
            let (instruction, consumed) = Self::parse_compiled_instruction(&data[offset..])?;
            instructions.push(instruction);
            offset += consumed;
        }

        // Parse address table lookups
        let mut address_table_lookups = Vec::new();
        if offset < data.len() {
            let num_lookups = data[offset] as usize;
            offset += 1;

            for _ in 0..num_lookups {
                let (lookup, consumed) = Self::parse_address_table_lookup(&data[offset..])?;
                address_table_lookups.push(lookup);
                offset += consumed;
            }
        }

        Ok(V0Message {
            header,
            account_keys,
            recent_blockhash,
            instructions,
            address_table_lookups,
        })
    }

    /// Parse compiled instruction from bytes
    fn parse_compiled_instruction(data: &[u8]) -> Result<(CompiledInstruction, usize)> {
        let mut offset = 0;
        
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing program_id_index".to_string()));
        }
        let program_id_index = data[offset];
        offset += 1;

        // Parse accounts length and indices
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing accounts length".to_string()));
        }
        let num_accounts = data[offset] as usize;
        offset += 1;

        if offset + num_accounts > data.len() {
            return Err(TerminatorError::SerializationError("Invalid accounts data".to_string()));
        }
        let accounts = data[offset..offset + num_accounts].to_vec();
        offset += num_accounts;

        // Parse instruction data length and data
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing instruction data length".to_string()));
        }
        let data_length = data[offset] as usize;
        offset += 1;

        if offset + data_length > data.len() {
            return Err(TerminatorError::SerializationError("Invalid instruction data".to_string()));
        }
        let instruction_data = data[offset..offset + data_length].to_vec();
        offset += data_length;

        Ok((CompiledInstruction {
            program_id_index,
            accounts,
            data: instruction_data,
        }, offset))
    }

    /// Parse address table lookup from bytes
    fn parse_address_table_lookup(data: &[u8]) -> Result<(MessageAddressTableLookup, usize)> {
        let mut offset = 0;
        
        // Parse account key
        if offset + 32 > data.len() {
            return Err(TerminatorError::SerializationError("Invalid lookup table key".to_string()));
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&data[offset..offset + 32]);
        let account_key = SolanaPubkey(key_bytes);
        offset += 32;

        // Parse writable indexes
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing writable indexes length".to_string()));
        }
        let num_writable = data[offset] as usize;
        offset += 1;

        if offset + num_writable > data.len() {
            return Err(TerminatorError::SerializationError("Invalid writable indexes".to_string()));
        }
        let writable_indexes = data[offset..offset + num_writable].to_vec();
        offset += num_writable;

        // Parse readonly indexes
        if offset >= data.len() {
            return Err(TerminatorError::SerializationError("Missing readonly indexes length".to_string()));
        }
        let num_readonly = data[offset] as usize;
        offset += 1;

        if offset + num_readonly > data.len() {
            return Err(TerminatorError::SerializationError("Invalid readonly indexes".to_string()));
        }
        let readonly_indexes = data[offset..offset + num_readonly].to_vec();
        offset += num_readonly;

        Ok((MessageAddressTableLookup {
            account_key,
            writable_indexes,
            readonly_indexes,
        }, offset))
    }

    /// Parse legacy transaction as versioned
    fn parse_legacy_versioned_transaction(data: &[u8]) -> Result<VersionedTransaction> {
        let legacy_tx: SolanaTransaction = bincode::deserialize(data)
            .map_err(|e| TerminatorError::SerializationError(format!("Failed to parse legacy transaction: {}", e)))?;
        
        Ok(VersionedTransaction {
            signatures: legacy_tx.signatures,
            message: VersionedMessage::Legacy(legacy_tx.message),
        })
    }

    /// Convert v0 message to legacy format by resolving lookup tables
    fn v0_to_legacy_message(v0_message: V0Message) -> Result<SolanaMessage> {
        let mut all_account_keys = v0_message.account_keys.clone();
        
        // For demo purposes, we'll create placeholder accounts for lookup table entries
        // In a real implementation, you'd resolve these from the blockchain state
        for lookup in &v0_message.address_table_lookups {
            // Add placeholder accounts for writable indexes
            for _ in &lookup.writable_indexes {
                all_account_keys.push(SolanaPubkey::new_unique());
            }
            // Add placeholder accounts for readonly indexes  
            for _ in &lookup.readonly_indexes {
                all_account_keys.push(SolanaPubkey::new_unique());
            }
        }

        // Update instructions to use the expanded account list
        let updated_instructions = v0_message.instructions;

        Ok(SolanaMessage {
            header: v0_message.header,
            account_keys: all_account_keys,
            recent_blockhash: v0_message.recent_blockhash,
            instructions: updated_instructions,
        })
    }

    /// Serialize transaction to Solana's wire format
    pub fn serialize_transaction(tx: &SolanaTransaction) -> Result<Vec<u8>> {
        bincode::serialize(tx)
            .map_err(|e| TerminatorError::SerializationError(format!("Failed to serialize transaction: {}", e)))
    }

    /// Parse transaction from JSON (like Solana RPC)
    pub fn parse_transaction_json(json: &str) -> Result<SolanaTransaction> {
        serde_json::from_str(json)
            .map_err(|e| TerminatorError::SerializationError(format!("Failed to parse JSON transaction: {}", e)))
    }

    /// Convert transaction to JSON
    pub fn transaction_to_json(tx: &SolanaTransaction) -> Result<String> {
        serde_json::to_string_pretty(tx)
            .map_err(|e| TerminatorError::SerializationError(format!("Failed to serialize JSON: {}", e)))
    }

    /// Create a simple transfer transaction in Solana format
    pub fn create_transfer_transaction(
        from: SolanaPubkey,
        to: SolanaPubkey,
        lamports: u64,
        recent_blockhash: SolanaHash,
    ) -> SolanaTransaction {
        // System program transfer instruction data
        let mut instruction_data = vec![2u8]; // Transfer instruction
        instruction_data.extend_from_slice(&lamports.to_le_bytes());

        let instruction = CompiledInstruction {
            program_id_index: 2, // System program will be at index 2
            accounts: vec![0, 1], // from=0, to=1
            data: instruction_data,
        };

        let message = SolanaMessage {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1, // system program
            },
            account_keys: vec![from, to, SolanaPubkey::system_program()],
            recent_blockhash,
            instructions: vec![instruction],
        };

        SolanaTransaction {
            signatures: vec![SolanaSignature([0u8; 64])], // Placeholder signature
            message,
        }
    }

    /// Extract message for signing (without signatures)
    pub fn message_data(message: &SolanaMessage) -> Result<Vec<u8>> {
        bincode::serialize(message)
            .map_err(|e| TerminatorError::SerializationError(format!("Failed to serialize message: {}", e)))
    }

    /// Validate transaction format
    pub fn validate_transaction_format(tx: &SolanaTransaction) -> Result<()> {
        // Check signature count matches required signatures
        if tx.signatures.len() != tx.message.header.num_required_signatures as usize {
            return Err(TerminatorError::TransactionExecutionFailed(
                "Signature count mismatch".to_string()
            ));
        }

        // Check account indices are valid
        let num_accounts = tx.message.account_keys.len() as u8;
        for instruction in &tx.message.instructions {
            if instruction.program_id_index >= num_accounts {
                return Err(TerminatorError::TransactionExecutionFailed(
                    "Invalid program_id_index".to_string()
                ));
            }
            
            for &account_index in &instruction.accounts {
                if account_index >= num_accounts {
                    return Err(TerminatorError::TransactionExecutionFailed(
                        "Invalid account index".to_string()
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Advanced Solana features with v0 support
pub struct SolanaFeatures;

impl SolanaFeatures {
    /// Create a Program Derived Address instruction
    pub fn create_pda_instruction(
        _program_id: SolanaPubkey,
        seeds: &[&[u8]],
        _payer: SolanaPubkey,
    ) -> Result<CompiledInstruction> {
        // Simplified PDA creation instruction
        let mut instruction_data = vec![0u8]; // CreateAccount instruction
        
        // Add seeds to instruction data
        instruction_data.push(seeds.len() as u8);
        for seed in seeds {
            instruction_data.push(seed.len() as u8);
            instruction_data.extend_from_slice(seed);
        }

        Ok(CompiledInstruction {
            program_id_index: 0, // Program ID should be resolved during compilation
            accounts: vec![0], // Payer account
            data: instruction_data,
        })
    }

    /// Parse Address Lookup Table (ALT) instruction
    pub fn parse_lookup_table_instruction(data: &[u8]) -> Result<Vec<SolanaPubkey>> {
        if data.len() < 32 {
            return Err(TerminatorError::SerializationError("Invalid ALT data".to_string()));
        }

        let mut addresses = Vec::new();
        let mut offset = 0;
        
        while offset + 32 <= data.len() {
            let mut addr = [0u8; 32];
            addr.copy_from_slice(&data[offset..offset + 32]);
            addresses.push(SolanaPubkey(addr));
            offset += 32;
        }

        Ok(addresses)
    }

    /// Check if transaction is v0 format
    pub fn is_v0_transaction(data: &[u8]) -> bool {
        !data.is_empty() && (data[0] & 0x80) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubkey_base58() {
        let pubkey = SolanaPubkey::new([1u8; 32]);
        let base58_str = pubkey.to_string();
        let parsed = SolanaPubkey::from_str(&base58_str).unwrap();
        assert_eq!(pubkey, parsed);
    }

    #[test]
    fn test_transaction_serialization() {
        let from = SolanaPubkey::new([1u8; 32]);
        let to = SolanaPubkey::new([2u8; 32]);
        let blockhash = SolanaHash([3u8; 32]);
        
        let tx = SolanaTransactionParser::create_transfer_transaction(
            from, to, 1000000, blockhash
        );

        // Test serialization round trip
        let serialized = SolanaTransactionParser::serialize_transaction(&tx).unwrap();
        let deserialized = SolanaTransactionParser::parse_transaction(&serialized).unwrap();
        
        assert_eq!(tx.message.account_keys.len(), deserialized.message.account_keys.len());
        assert_eq!(tx.message.instructions.len(), deserialized.message.instructions.len());
    }

    #[test]
    fn test_transaction_validation() {
        let from = SolanaPubkey::new([1u8; 32]);
        let to = SolanaPubkey::new([2u8; 32]);
        let blockhash = SolanaHash([3u8; 32]);
        
        let tx = SolanaTransactionParser::create_transfer_transaction(
            from, to, 1000000, blockhash
        );

        let result = SolanaTransactionParser::validate_transaction_format(&tx);
        assert!(result.is_ok(), "Valid transaction should pass validation");
    }

    #[test]
    fn test_v0_transaction_detection() {
        let v0_data = vec![0x81, 0x00]; // v0 transaction with 1 signature
        let legacy_data = vec![0x01, 0x00]; // Legacy transaction with 1 signature
        
        assert!(SolanaFeatures::is_v0_transaction(&v0_data));
        assert!(!SolanaFeatures::is_v0_transaction(&legacy_data));
    }

    #[test]
    fn test_json_serialization() {
        let from = SolanaPubkey::new([1u8; 32]);
        let to = SolanaPubkey::new([2u8; 32]);
        let blockhash = SolanaHash([3u8; 32]);
        
        let tx = SolanaTransactionParser::create_transfer_transaction(
            from, to, 1000000, blockhash
        );

        let json = SolanaTransactionParser::transaction_to_json(&tx).unwrap();
        let parsed = SolanaTransactionParser::parse_transaction_json(&json).unwrap();
        
        assert_eq!(tx.message.account_keys.len(), parsed.message.account_keys.len());
    }

    #[test]
    fn test_system_program_ids() {
        let system = SolanaPubkey::system_program();
        let token = SolanaPubkey::token_program();
        
        assert_eq!(system.0, [0u8; 32]);
        assert_ne!(system, token);
    }
} 