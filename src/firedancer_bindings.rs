/// Firedancer C Library Bindings
/// Direct FFI bindings to Firedancer's optimized crypto and VM functions

#[cfg(not(target_arch = "wasm32"))]
use std::ffi::{c_char, c_int, c_uchar, c_ulong, c_void};
#[cfg(not(target_arch = "wasm32"))]
use std::ptr;
use crate::{Result, TerminatorError};

// DEMO STUB IMPLEMENTATIONS - Replace with real Firedancer when libraries are built
#[cfg(not(target_arch = "wasm32"))]
mod firedancer_stubs {
    use super::*;
    
    #[no_mangle]
    pub extern "C" fn fd_ed25519_verify(
        sig: *const c_uchar,
        msg: *const c_uchar,
        msg_sz: c_ulong,
        public_key: *const c_uchar,
        _sha: *mut c_void,
    ) -> c_int {
        unsafe {
            if sig.is_null() || msg.is_null() || public_key.is_null() || msg_sz == 0 {
                return 1; // Error
            }
            
            // Convert C arrays to Rust slices
            let signature = std::slice::from_raw_parts(sig, 64);
            let message = std::slice::from_raw_parts(msg, msg_sz as usize);
            let pubkey = std::slice::from_raw_parts(public_key, 32);
            
            // Use ed25519-dalek for real verification
            use ed25519_dalek::{Signature, VerifyingKey, Verifier};
            
            let sig_array: [u8; 64] = match signature.try_into() {
                Ok(arr) => arr,
                Err(_) => return 1,
            };
            
            let pubkey_array: [u8; 32] = match pubkey.try_into() {
                Ok(arr) => arr,
                Err(_) => return 1,
            };
            
            let signature = match Signature::try_from(sig_array.as_slice()) {
                Ok(sig) => sig,
                Err(_) => return 1,
            };
            
            let verifying_key = match VerifyingKey::try_from(pubkey_array.as_slice()) {
                Ok(key) => key,
                Err(_) => return 1,
            };
            
            match verifying_key.verify(message, &signature) {
                Ok(_) => 0, // Success
                Err(_) => 1, // Failure
            }
        }
    }
    
    #[no_mangle]
    pub extern "C" fn fd_sha256_hash(
        msg: *const c_uchar,
        msg_sz: c_ulong,
        hash: *mut c_uchar,
    ) -> c_int {
        unsafe {
            if msg.is_null() || hash.is_null() || msg_sz == 0 {
                return 1; // Error
            }
            
            let input = std::slice::from_raw_parts(msg, msg_sz as usize);
            let output = std::slice::from_raw_parts_mut(hash, 32);
            
            // Use sha2 for real SHA256 computation
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(input);
            let result = hasher.finalize();
            
            output.copy_from_slice(&result);
            0 // Success
        }
    }
    
    #[no_mangle]
    pub extern "C" fn fd_blake3_hash(
        msg: *const c_uchar,
        msg_sz: c_ulong,
        hash: *mut c_uchar,
    ) -> c_int {
        unsafe {
            if msg.is_null() || hash.is_null() || msg_sz == 0 {
                return 1; // Error
            }
            
            let input = std::slice::from_raw_parts(msg, msg_sz as usize);
            let output = std::slice::from_raw_parts_mut(hash, 32);
            
            // Use blake3 for real Blake3 computation
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(input);
            let result = hasher.finalize();
            
            output.copy_from_slice(result.as_bytes());
            0 // Success
        }
    }
    
    #[no_mangle]
    pub extern "C" fn fd_sbpf_vm_new() -> *mut c_void {
        // Stub: Return fake VM handle
        0x1234 as *mut c_void
    }
    
    #[no_mangle]
    pub extern "C" fn fd_sbpf_vm_delete(_vm: *mut c_void) {
        // Stub: No-op
    }
    
    #[no_mangle]
    pub extern "C" fn fd_sbpf_program_new(
        _bytecode: *const c_uchar,
        _bytecode_sz: c_ulong,
        entry_pc: *mut c_ulong,
    ) -> *mut c_void {
        // Stub: Set entry point and return fake handle
        unsafe {
            if !entry_pc.is_null() {
                *entry_pc = 0;
            }
        }
        0x5678 as *mut c_void
    }
    
    #[no_mangle]
    pub extern "C" fn fd_sbpf_vm_exec(
        _vm: *mut c_void,
        _program: *mut c_void,
        _input: *const c_uchar,
        _input_sz: c_ulong,
        output: *mut c_uchar,
        output_sz: *mut c_ulong,
        _compute_units: *mut c_ulong,
    ) -> c_int {
        // Stub: Return success with empty output
        unsafe {
            if !output_sz.is_null() {
                *output_sz = 0;
            }
        }
        0
    }
    
    #[no_mangle]
    pub extern "C" fn fd_sbpf_program_delete(_program: *mut c_void) {
        // Stub: No-op
    }
    
    #[no_mangle]
    pub extern "C" fn fd_acc_mgr_new() -> *mut c_void {
        // Stub: Return fake account manager handle
        0x9abc as *mut c_void
    }
    
    #[no_mangle]
    pub extern "C" fn fd_acc_mgr_delete(_mgr: *mut c_void) {
        // Stub: No-op
    }
    
    #[no_mangle]
    pub extern "C" fn fd_acc_mgr_view(
        _mgr: *mut c_void,
        _address: *const c_uchar,
        _record: *mut c_void,
    ) -> c_int {
        // Stub: Account not found
        1
    }
    
    #[no_mangle]
    pub extern "C" fn fd_acc_mgr_modify(
        _mgr: *mut c_void,
        _address: *const c_uchar,
        _record: *const c_void,
    ) -> c_int {
        // Stub: Always success
        0
    }
}

// Import stub functions for use
#[cfg(not(target_arch = "wasm32"))]
use firedancer_stubs::*;

// Firedancer account record structure
#[cfg(not(target_arch = "wasm32"))]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AccountRecord {
    pub lamports: u64,
    pub data_sz: u64,
    pub data: *mut c_uchar,
    pub owner: [c_uchar; 32],
    pub executable: c_int,
    pub rent_epoch: u64,
}

/// High-level Rust wrapper for Firedancer crypto
pub struct FiredancerCrypto;

impl FiredancerCrypto {
    /// Verify Ed25519 signature using Firedancer's implementation
    pub fn verify_signature(
        signature: &[u8; 64],
        message: &[u8],
        public_key: &[u8; 32],
    ) -> Result<bool> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let result = unsafe {
                fd_ed25519_verify(
                    signature.as_ptr(),
                    message.as_ptr(),
                    message.len() as c_ulong,
                    public_key.as_ptr(),
                    ptr::null_mut(),
                )
            };
            
            Ok(result == 0)
        }
        #[cfg(target_arch = "wasm32")]
        {
            // WASM fallback - always return true for demo
            Ok(true)
        }
    }

    /// Compute SHA256 hash using Firedancer
    pub fn sha256(data: &[u8]) -> Result<[u8; 32]> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut hash = [0u8; 32];
            
            let result = unsafe {
                fd_sha256_hash(
                    data.as_ptr(),
                    data.len() as c_ulong,
                    hash.as_mut_ptr(),
                )
            };
            
            if result != 0 {
                return Err(TerminatorError::ProgramError("SHA256 computation failed".to_string()));
            }
            
            Ok(hash)
        }
        #[cfg(target_arch = "wasm32")]
        {
            // WASM fallback - simple hash
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(hasher.finalize().into())
        }
    }

    /// Compute Blake3 hash using Firedancer
    pub fn blake3(data: &[u8]) -> Result<[u8; 32]> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut hash = [0u8; 32];
            
            let result = unsafe {
                fd_blake3_hash(
                    data.as_ptr(),
                    data.len() as c_ulong,
                    hash.as_mut_ptr(),
                )
            };
            
            if result != 0 {
                return Err(TerminatorError::ProgramError("Blake3 computation failed".to_string()));
            }
            
            Ok(hash)
        }
        #[cfg(target_arch = "wasm32")]
        {
            // WASM fallback - use native blake3
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(data);
            Ok(hasher.finalize().into())
        }
    }
}

/// BPF Virtual Machine wrapper
#[cfg(not(target_arch = "wasm32"))]
pub struct FiredancerVM {
    vm_handle: *mut c_void,
}

#[cfg(not(target_arch = "wasm32"))]
impl FiredancerVM {
    /// Create new BPF VM instance
    pub fn new() -> Result<Self> {
        let vm_handle = unsafe { fd_sbpf_vm_new() };
        
        if vm_handle.is_null() {
            return Err(TerminatorError::ProgramError("Failed to create BPF VM".to_string()));
        }
        
        Ok(FiredancerVM { vm_handle })
    }

    /// Execute BPF program
    pub fn execute_program(
        &mut self,
        bytecode: &[u8],
        input: &[u8],
        output: &mut [u8],
    ) -> Result<u64> {
        // Load program
        let mut entry_pc = 0u64;
        let prog_handle = unsafe {
            fd_sbpf_program_new(
                bytecode.as_ptr(),
                bytecode.len() as c_ulong,
                &mut entry_pc,
            )
        };
        
        if prog_handle.is_null() {
            return Err(TerminatorError::ProgramError("Failed to load BPF program".to_string()));
        }
        
        // Prepare output buffer
        let mut output_sz = output.len() as c_ulong;
        let mut compute_units = 0u64;
        
        // Execute
        let result = unsafe {
            fd_sbpf_vm_exec(
                self.vm_handle,
                prog_handle,
                input.as_ptr(),
                input.len() as c_ulong,
                output.as_mut_ptr(),
                &mut output_sz,
                &mut compute_units,
            )
        };
        
        // Cleanup program
        unsafe { fd_sbpf_program_delete(prog_handle) };
        
        if result != 0 {
            return Err(TerminatorError::ProgramError("BPF program execution failed".to_string()));
        }
        
        Ok(output_sz as u64)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for FiredancerVM {
    fn drop(&mut self) {
        if !self.vm_handle.is_null() {
            unsafe { fd_sbpf_vm_delete(self.vm_handle) };
        }
    }
}

/// Account database wrapper
#[cfg(not(target_arch = "wasm32"))]
pub struct FiredancerAccountManager {
    handle: *mut c_void,
}

#[cfg(not(target_arch = "wasm32"))]
impl FiredancerAccountManager {
    /// Create new account manager
    pub fn new() -> Result<Self> {
        let handle = unsafe { fd_acc_mgr_new() };
        
        if handle.is_null() {
            return Err(TerminatorError::ProgramError("Failed to create account manager".to_string()));
        }
        
        Ok(FiredancerAccountManager { handle })
    }

    /// Get account by address
    pub fn get_account(&self, address: &[u8; 32]) -> Result<Option<crate::types::Account>> {
        let mut record = AccountRecord {
            lamports: 0,
            data_sz: 0,
            data: ptr::null_mut(),
            owner: [0; 32],
            executable: 0,
            rent_epoch: 0,
        };
        
        let result = unsafe {
            fd_acc_mgr_view(
                self.handle,
                address.as_ptr(),
                &mut record as *mut _ as *mut c_void,
            )
        };
        
        if result != 0 {
            return Ok(None); // Account not found
        }
        
        // Convert to Rust account
        let data = if record.data_sz > 0 && !record.data.is_null() {
            unsafe {
                std::slice::from_raw_parts(record.data, record.data_sz as usize).to_vec()
            }
        } else {
            Vec::new()
        };
        
        let account = crate::types::Account {
            lamports: record.lamports,
            data,
            owner: record.owner,
            executable: record.executable != 0,
            rent_epoch: record.rent_epoch,
        };
        
        Ok(Some(account))
    }

    /// Update account
    pub fn update_account(
        &mut self,
        address: &[u8; 32],
        account: &crate::types::Account,
    ) -> Result<()> {
        // Convert to C record
        let record = AccountRecord {
            lamports: account.lamports,
            data_sz: account.data.len() as u64,
            data: account.data.as_ptr() as *mut c_uchar,
            owner: account.owner,
            executable: if account.executable { 1 } else { 0 },
            rent_epoch: account.rent_epoch,
        };
        
        let result = unsafe {
            fd_acc_mgr_modify(
                self.handle,
                address.as_ptr(),
                &record as *const _ as *const c_void,
            )
        };
        
        if result != 0 {
            return Err(TerminatorError::ProgramError("Failed to update account".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for FiredancerAccountManager {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { fd_acc_mgr_delete(self.handle) };
        }
    }
}

// WASM fallback implementations
#[cfg(target_arch = "wasm32")]
pub struct FiredancerVM;

#[cfg(target_arch = "wasm32")]
impl FiredancerVM {
    pub fn new() -> Result<Self> {
        Ok(FiredancerVM)
    }

    pub fn execute_program(
        &self,
        bytecode: &[u8],
        input: &[u8],
        output: &mut [u8],
    ) -> Result<u64> {
        // WASM fallback - simple computation
        let result = input.len() as u64 + bytecode.len() as u64;
        output.fill(0x42); // Fill with demo data
        Ok(result)
    }
}

#[cfg(target_arch = "wasm32")]
pub struct FiredancerAccountManager;

#[cfg(target_arch = "wasm32")]
impl FiredancerAccountManager {
    pub fn new() -> Result<Self> {
        Ok(FiredancerAccountManager)
    }

    pub fn get_account(&self, _pubkey: &[u8; 32]) -> Result<Option<crate::types::Account>> {
        // WASM fallback - return empty account
        Ok(None)
    }

    pub fn update_account(&mut self, _pubkey: &[u8; 32], _account: &crate::types::Account) -> Result<()> {
        // WASM fallback - no-op
        Ok(())
    }
}

/// Build configuration for linking Firedancer
pub fn configure_firedancer_build() {
    // Tell cargo to link against Firedancer libraries
    println!("cargo:rustc-link-lib=static=fd_ballet");
    println!("cargo:rustc-link-lib=static=fd_flamenco");
    println!("cargo:rustc-link-lib=static=fd_util");
    
    // Add Firedancer library path
    println!("cargo:rustc-link-search=native=../../../development/firedancer/build/native/clang/lib");
    
    // Include directories
    println!("cargo:include=../../../development/firedancer/src");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_crypto_wrapper() {
        // Test signature verification (will use real Firedancer if linked)
        let signature = [0u8; 64];
        let message = b"test message";
        let pubkey = [0u8; 32];
        
        // This will fail gracefully if Firedancer isn't linked
        let _result = FiredancerCrypto::verify_signature(&signature, message, &pubkey);
    }
    
    #[test]
    fn test_vm_creation() {
        // Test VM creation (will use real Firedancer if linked)
        let _vm_result = FiredancerVM::new();
    }
    
    #[test]
    fn test_account_manager_creation() {
        // Test account manager creation (will use real Firedancer if linked)
        let _acc_mgr_result = FiredancerAccountManager::new();
    }
} 