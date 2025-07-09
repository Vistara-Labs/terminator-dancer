/// DEBUG INSTRUCTION DATA
/// ======================
/// 
/// Find exactly where the lamport amount is encoded and why we're getting wrong numbers

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

fn main() {
    println!("🔍 DEBUGGING INSTRUCTION DATA");
    println!("=============================");
    println!("Block explorer shows: 2,034,200 lamports (0.002 SOL)");
    println!("Let's find where this is encoded...");
    println!();

    let base64_data = "AWDBlrdyFjzjDgf9gWioXrCB/YJpHeENZcIEwNPzflGviVkElIKpUR7yvnwrNsz0cuq5MGm0FlR/7gf8piruIw6AAQABA/NGAeBeYMRrJvmYo4E2q+pEKIVjl40S0g00e/NP8G7JAGBZvnD3SSIz2B5EgB+fk5vSvVThak5kIyxG8n1zLKIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVYxfd1NZLpMnJgbaVBboof2ZjR+cEKxQwMiWhlFusxAQICAAEMAgAAAGgKHwAAAAAAAA==";

    let tx_bytes = BASE64.decode(base64_data).expect("Valid base64");
    
    // Jump to offset 66 where we know message starts
    let mut offset = 66;
    
    // Skip header (3 bytes) + account count (1 byte) = 4 bytes
    offset += 4;
    
    // Skip 3 accounts (3 * 32 = 96 bytes)
    offset += 96;
    
    // Skip blockhash (32 bytes)
    offset += 32;
    
    // Now at instruction count
    println!("📍 At instruction count, offset {}: {}", offset, tx_bytes[offset]);
    offset += 1;
    
    // Instruction 1
    println!("📍 Program ID index: {}", tx_bytes[offset]);
    offset += 1;
    
    println!("📍 Accounts in instruction: {}", tx_bytes[offset]);
    let accounts_in_instruction = tx_bytes[offset] as usize;
    offset += 1;
    
    // Skip account indices
    for i in 0..accounts_in_instruction {
        println!("📍 Account index {}: {}", i, tx_bytes[offset]);
        offset += 1;
    }
    
    // Data length
    println!("📍 Data length: {}", tx_bytes[offset]);
    let data_length = tx_bytes[offset] as usize;
    offset += 1;
    
    // Instruction data
    println!("📍 Raw instruction data ({} bytes):", data_length);
    let instruction_data = &tx_bytes[offset..offset + data_length];
    
    for (i, byte) in instruction_data.iter().enumerate() {
        print!("{:02x} ", byte);
        if i % 8 == 7 { println!(); }
    }
    println!();
    
    if instruction_data.len() >= 9 {
        println!("📊 DECODING:");
        println!("   Byte 0 (instruction type): {} = {}", instruction_data[0], 
                 if instruction_data[0] == 2 { "Transfer" } else { "Unknown" });
        
        if instruction_data[0] == 2 {
            // Try different endianness and see what we get
            println!();
            println!("🔍 TRYING DIFFERENT INTERPRETATIONS:");
            
            // Little endian (should be correct for Solana)
            let lamports_le = u64::from_le_bytes([
                instruction_data[1], instruction_data[2], instruction_data[3], instruction_data[4],
                instruction_data[5], instruction_data[6], instruction_data[7], instruction_data[8],
            ]);
            println!("   Little endian: {} lamports = {:.9} SOL", lamports_le, lamports_le as f64 / 1_000_000_000.0);
            
            // Big endian
            let lamports_be = u64::from_be_bytes([
                instruction_data[1], instruction_data[2], instruction_data[3], instruction_data[4],
                instruction_data[5], instruction_data[6], instruction_data[7], instruction_data[8],
            ]);
            println!("   Big endian: {} lamports = {:.9} SOL", lamports_be, lamports_be as f64 / 1_000_000_000.0);
            
            // Try just the first 4 bytes as u32
            let lamports_u32_le = u32::from_le_bytes([
                instruction_data[1], instruction_data[2], instruction_data[3], instruction_data[4],
            ]) as u64;
            println!("   First 4 bytes as u32 LE: {} lamports = {:.9} SOL", lamports_u32_le, lamports_u32_le as f64 / 1_000_000_000.0);
            
            // Show the raw bytes we're working with
            println!();
            println!("🔍 RAW LAMPORT BYTES:");
            for i in 1..9 {
                println!("   Byte {}: 0x{:02x} = {}", i, instruction_data[i], instruction_data[i]);
            }
            
            // Check if 2,034,200 appears anywhere
            let target = 2034200u64;
            println!();
            println!("🎯 LOOKING FOR 2,034,200 lamports:");
            println!("   Target as LE bytes: {:02x?}", target.to_le_bytes());
            println!("   Target as BE bytes: {:02x?}", target.to_be_bytes());
            
            if lamports_le == target {
                println!("   ✅ FOUND IT! Little endian matches block explorer");
            } else if lamports_be == target {
                println!("   ✅ FOUND IT! Big endian matches block explorer");
            } else if lamports_u32_le == target {
                println!("   ✅ FOUND IT! First 4 bytes as u32 matches block explorer");
            } else {
                println!("   ❌ None of these match the block explorer amount");
                println!("   🤔 Maybe the data is encoded differently or at different offset?");
            }
        }
    }
    
    println!();
} 