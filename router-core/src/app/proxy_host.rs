
/// Extract HTTP Host header directly from byte buffer without string conversion
/// Uses optimized byte-level parsing for better performance
pub fn extract_http_host(buf: &[u8], length: usize) -> Option<String> {
    let max_scan_len = std::cmp::min(length, 1024);
    let host_pattern = b"host:";
    
    // Search for "host:" header (case-insensitive)
    for i in 0..max_scan_len - host_pattern.len() {
        if &buf[i..i+5].to_ascii_lowercase() == host_pattern {
            // Found the Host header, now extract the value
            let start_idx = i + 5;
            let mut end_idx = start_idx;
            
            // Find the end of line
            while end_idx < max_scan_len && buf[end_idx] != b'\r' && buf[end_idx] != b'\n' {
                end_idx += 1;
            }
            
            // Convert the host value to a String, trimming whitespace
            if end_idx > start_idx {
                let host_bytes = &buf[start_idx..end_idx];
                // Trim leading whitespace
                let mut trim_start = 0;
                while trim_start < host_bytes.len() && (host_bytes[trim_start] == b' ' || host_bytes[trim_start] == b'\t') {
                    trim_start += 1;
                }
                
                // Trim trailing whitespace
                let mut trim_end = host_bytes.len();
                while trim_end > trim_start && (host_bytes[trim_end-1] == b' ' || host_bytes[trim_end-1] == b'\t') {
                    trim_end -= 1;
                }
                
                if trim_end > trim_start {
                    return std::str::from_utf8(&host_bytes[trim_start..trim_end]).ok().map(String::from);
                }
            }
            
            break;
        }
    }
    
    None
}
