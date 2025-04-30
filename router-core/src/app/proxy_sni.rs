
/// Optimized SNI extraction function that uses direct byte access and caching
/// This is faster than the original extract_sni function
pub fn extract_sni_fast(buf: &[u8]) -> Option<String> {
    // Quick check for TLS handshake
    if buf.len() < 5 || buf[0] != 0x16 {
        return None;
    }
    
    // Use a more direct approach to find the SNI extension
    // TLS record header is 5 bytes, followed by handshake message
    // Skip to the extensions section directly based on the structure
    let mut pos = 5; // Skip TLS record header
    
    if pos + 4 > buf.len() {
        return None;
    }
    
    // Skip handshake type (1 byte) and length (3 bytes)
    pos += 4;
    
    if pos + 2 > buf.len() {
        return None;
    }
    
    // Skip client version (2 bytes)
    pos += 2;
    
    if pos + 32 > buf.len() {
        return None;
    }
    
    // Skip client random (32 bytes)
    pos += 32;
    
    if pos + 1 > buf.len() {
        return None;
    }
    
    // Get session ID length and skip session ID
    let session_id_len = buf[pos] as usize;
    pos += 1;
    
    if pos + session_id_len > buf.len() {
        return None;
    }
    
    pos += session_id_len;
    
    if pos + 2 > buf.len() {
        return None;
    }
    
    // Get cipher suites length and skip cipher suites
    let cipher_suites_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
    pos += 2;
    
    if pos + cipher_suites_len > buf.len() {
        return None;
    }
    
    pos += cipher_suites_len;
    
    if pos + 1 > buf.len() {
        return None;
    }
    
    // Get compression methods length and skip compression methods
    let compression_methods_len = buf[pos] as usize;
    pos += 1;
    
    if pos + compression_methods_len > buf.len() {
        return None;
    }
    
    pos += compression_methods_len;
    
    if pos + 2 > buf.len() {
        return None;
    }
    
    // Get extensions length
    let extensions_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
    pos += 2;
    
    if pos + extensions_len > buf.len() {
        return None;
    }
    
    // Process extensions
    let extensions_end = pos + extensions_len;
    while pos + 4 <= extensions_end {
        let ext_type = ((buf[pos] as u16) << 8) | (buf[pos + 1] as u16);
        let ext_len = ((buf[pos + 2] as usize) << 8) | (buf[pos + 3] as usize);
        pos += 4;
        
        if pos + ext_len > extensions_end {
            break;
        }
        
        // SNI extension type is 0
        if ext_type == 0 {
            // Parse SNI extension
            if ext_len >= 2 {
                let sni_list_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
                pos += 2;
                
                if pos + sni_list_len <= extensions_end && sni_list_len >= 3 {
                    // Name type (should be 0 for hostname)
                    if buf[pos] == 0 {
                        pos += 1;
                        
                        // Hostname length
                        let hostname_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
                        pos += 2;
                        
                        if pos + hostname_len <= extensions_end {
                            // Extract hostname
                            return std::str::from_utf8(&buf[pos..pos + hostname_len]).ok().map(String::from);
                        }
                    }
                }
            }
            
            break;
        }
        
        pos += ext_len;
    }
    
    None
}

/// # Find SNI extension in TLS Client Hello
///
/// Helper function that searches for the SNI extension within a TLS Client Hello message.
/// The SNI extension is identified by type 0x0000 in the extensions section of a
/// Client Hello message.
///
/// ## TLS Extensions Format
/// Extensions appear at the end of the Client Hello message after:
/// - Content Type (1 byte)
/// - TLS Version (2 bytes)
/// - Record Length (2 bytes)
/// - Handshake Type (1 byte)
/// - Handshake Length (3 bytes)
/// - TLS Version (2 bytes)
/// - Random (32 bytes)
/// - Session ID (variable, length indicated by 1 byte)
/// - Cipher Suites (variable, length indicated by 2 bytes)
/// - Compression Methods (variable, length indicated by 1 byte)
/// - Extensions Length (2 bytes)
///
/// Each extension has:
/// - Extension Type (2 bytes, 0x0000 for SNI)
/// - Extension Length (2 bytes)
/// - Extension Data (variable)
///
/// ## SNI Extension Structure
/// For SNI specifically:
/// - Extension Type: 0x0000
/// - Extension Length: 2 + server_name_list_length
/// - Server Name List Length: 2 bytes
/// - Name Type: 1 byte (0x00 for hostname)
/// - Hostname Length: 2 bytes
/// - Hostname: UTF-8 encoded string
///
/// ## Parameters
/// * `buf` - The raw bytes from the TLS handshake
///
/// ## Returns
/// * `Some(usize)` - Position where the SNI hostname length data begins
/// * `None` - If SNI extension could not be found
///
/// ## Implementation Details
/// This is a simplified implementation that:
/// 1. Looks for byte patterns that might indicate an SNI extension
/// 2. Does not fully parse the TLS record structure
/// 3. May return false positives or miss the extension
///
/// In a production environment, a formal TLS parser should be used.
fn find_sni_extension(buf: &[u8]) -> Option<usize> {
    // Minimum TLS Client Hello with SNI should be at least 45 bytes
    // (5 byte record header + 4 byte handshake header + 2 byte version + 32 byte random +
    //  1 byte session ID length + 2 byte cipher suites length + 1 byte compression methods length +
    //  2 byte extensions length + 4 byte SNI extension header + 2 byte server name list length +
    //  1 byte name type + 2 byte hostname length)
    if buf.len() < 45 {
        return None;
    }

    // This simplified implementation looks for the SNI extension pattern:
    // - Extension Type 0x0000 (SNI)
    // - Followed by a length field
    // - Followed by server name list length
    // - Followed by name type 0x00 (hostname)

    // Search through the buffer for potential SNI extension
    // We're looking for the pattern: 0x00 0x00 (extension type) followed by length bytes
    // and then a server name list that starts with 0x00 (hostname indicator)
    for i in 0..buf.len() - 8 {
        // Possible SNI extension pattern:
        // 0x00 0x00 (extension type) followed by length bytes and name type 0x00
        if buf[i] == 0x00 && buf[i + 1] == 0x00 && buf[i + 4] == 0x00 {
            // Extract extension length
            let ext_len = ((buf[i + 2] as usize) << 8) | (buf[i + 3] as usize);

            // Sanity check the length
            if ext_len > 0 && ext_len < 1000 && i + 4 + ext_len <= buf.len() {
                // The actual hostname length starts 3 bytes after the name type
                return Some(i + 5);
            }
        }
    }

    // Fallback to the original simplified method which may catch some cases
    // that the more specific pattern above misses
    for i in 0..buf.len() - 4 {
        if buf[i] == 0x00 && buf[i + 1] == 0x00 && buf[i + 2] == 0x00 {
            return Some(i + 3);
        }
    }

    None
}