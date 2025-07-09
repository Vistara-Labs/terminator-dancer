/// TRANSACTION BYTE DEBUGGER
/// =========================
/// 
/// inspection of the exact byte structure to find parsing issues

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

fn main() {
    println!("🔍 TRANSACTION BYTE DEBUGGER");
    println!("============================");
    
    let base64_data = "AWDBlrdyFjzjDgf9gWioXrCB/YJpHeENZcIEwNPzflGviVkElIKpUR7yvnwrNsz0cuq5MGm0FlR/7gf8piruIw6AAQABA/NGAeBeYMRrJvmYo4E2q+pEKIVjl40S0g00e/NP8G7JAGBZvnD3SSIz2B5EgB+fk5vSvVThak5kIyxG8n1zLKIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVYxfd1NZLpMnJgbaVBboof2ZjR+cEKxQwMiWhlFusxAQICAAEMAgAAAGgKHwAAAAAAAA==";
    
    let tx_bytes = BASE64.decode(base64_data).expect("Valid base64");
    
    println!("📊 Total length: {} bytes", tx_bytes.len());
    println!("🔢 Raw bytes (hex):");
    
    // Print all bytes in hex with offset
    for (i, chunk) in tx_bytes.chunks(16).enumerate() {
        print!("{:04x}: ", i * 16);
        for (j, &byte) in chunk.iter().enumerate() {
            if j == 8 { print!(" "); }
            print!("{:02x} ", byte);
        }
        
        // Pad if chunk is less than 16 bytes
        for j in chunk.len()..16 {
            if j == 8 { print!(" "); }
            print!("   ");
        }
        
        print!(" |");
        for &byte in chunk {
            let ch = if byte >= 32 && byte <= 126 { byte as char } else { '.' };
            print!("{}", ch);
        }
        println!("|");
    }
    
    println!();
    println!("🧪 STRUCTURE ANALYSIS:");
    println!("=====================");
    
    analyze_structure(&tx_bytes);
}

fn analyze_structure(bytes: &[u8]) {
    println!("📋 Byte 0: 0x{:02x} ({}) - Signature count", bytes[0], bytes[0]);
    
    let num_sigs = bytes[0] as usize;
    let sigs_end = 1 + (num_sigs * 64);
    
    println!("🔐 Signatures: {} (bytes 1-{})", num_sigs, sigs_end);
    
    if sigs_end < bytes.len() {
        println!("📨 Message starts at byte {}", sigs_end);
        println!("📋 Message first bytes: {:02x} {:02x} {:02x}", 
                 bytes[sigs_end], bytes[sigs_end + 1], bytes[sigs_end + 2]);
        
        // Check if there might be a version byte before the message
        if sigs_end > 1 && bytes[sigs_end] > 64 {
            println!("⚠️  Suspicious first message byte: {} (too high for required_signatures)", bytes[sigs_end]);
            
            // Check if there's a length or version field
            println!("🔍 Checking for possible structure variations:");
            
            // Maybe there's a version/format marker?
            for offset in 0..4 {
                if sigs_end + offset + 3 < bytes.len() {
                    println!("   Option {}: Message at +{}: {:02x} {:02x} {:02x}", 
                             offset + 1, offset,
                             bytes[sigs_end + offset], 
                             bytes[sigs_end + offset + 1], 
                             bytes[sigs_end + offset + 2]);
                }
            }
            
            // Check for Solana transaction version (compact encoding)
            println!();
            println!("🔧 COMPACT ENCODING CHECK:");
            println!("   Checking if message uses compact-array encoding...");
            
            // In Solana wire format, arrays can be compact-encoded
            let mut offset = sigs_end;
            
            // Try to read compact-u16 for message header
            if let Some((value, consumed)) = read_compact_u16(&bytes[offset..]) {
                println!("   Compact value at {}: {} (consumed {} bytes)", offset, value, consumed);
                offset += consumed;
                
                if offset + 2 < bytes.len() {
                    println!("   Next bytes: {:02x} {:02x} {:02x}", 
                             bytes[offset], bytes[offset + 1], bytes[offset + 2]);
                }
            }
        }
        
        println!();
        println!("🎯 LIKELY CORRECT STRUCTURE:");
        try_correct_parsing(&bytes[sigs_end..]);
    }
}

fn read_compact_u16(data: &[u8]) -> Option<(u16, usize)> {
    if data.is_empty() {
        return None;
    }
    
    let first_byte = data[0];
    
    if first_byte < 0x80 {
        // Single byte encoding
        Some((first_byte as u16, 1))
    } else if data.len() >= 2 {
        // Two byte encoding
        let value = ((first_byte & 0x7F) as u16) | ((data[1] as u16) << 7);
        Some((value, 2))
    } else {
        None
    }
}

fn try_correct_parsing(message_bytes: &[u8]) {
    println!("Attempting to find correct message structure...");
    
    // The message might be directly encoded without additional framing
    // Let's try to find reasonable header values
    
    for start_offset in 0..8.min(message_bytes.len()) {
        if start_offset + 3 >= message_bytes.len() {
            break;
        }
        
        let header = &message_bytes[start_offset..start_offset + 3];
        let req_sigs = header[0];
        let ro_signed = header[1]; 
        let ro_unsigned = header[2];
        
        // Check if these look like reasonable values
        if req_sigs <= 16 && ro_signed <= 16 && ro_unsigned <= 16 {
            println!("✅ FOUND REASONABLE HEADER at offset +{}:", start_offset);
            println!("   Required signatures: {}", req_sigs);
            println!("   Readonly signed: {}", ro_signed);
            println!("   Readonly unsigned: {}", ro_unsigned);
            
            let mut offset = start_offset + 3;
            if offset < message_bytes.len() {
                let account_count = message_bytes[offset];
                println!("   Account count: {}", account_count);
                
                if account_count <= 32 {
                    println!("   ✅ This looks like a valid transaction structure!");
                    break;
                }
            }
        }
    }
} 