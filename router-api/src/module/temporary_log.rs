use chrono::TimeZone;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque}; // Added HashSet to imports
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::ptr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogStoreError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),
}

// Enum for selecting bytes metric type
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum BytesMetric {
    BytesIn,
    BytesOut,
    BytesTotal,
}

#[derive(Debug)] // Added Debug for logging in append_data
pub struct TemporaryLog {
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub status_code: i32,
    pub peer: (String, String),
    pub conn_id: String,
    pub conn_type: String,
    pub conn_req: i8,   // 1 indicate connection in
    pub conn_res: i8,   // 1 indicate connection dirupted
    pub bytes_in: i32,  // bytes in
    pub bytes_out: i32, // bytes out
}

impl bincode::enc::Encode for TemporaryLog {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let timestamp = self.date_time.timestamp();
        let nanos = self.date_time.timestamp_subsec_nanos();
        timestamp.encode(encoder)?;
        nanos.encode(encoder)?;
        self.status_code.encode(encoder)?;
        self.peer.0.encode(encoder)?;
        self.peer.1.encode(encoder)?;
        self.conn_id.encode(encoder)?;
        self.conn_type.encode(encoder)?;
        self.conn_req.encode(encoder)?;
        self.conn_res.encode(encoder)?;
        self.bytes_in.encode(encoder)?;
        self.bytes_out.encode(encoder)?;
        Ok(())
    }
}

impl bincode::de::Decode<()> for TemporaryLog {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let timestamp: i64 = i64::decode(decoder)?;
        let nanos: u32 = u32::decode(decoder)?;
        let date_time = match Utc.timestamp_opt(timestamp, nanos) {
            chrono::LocalResult::Single(dt) => dt,
            _ => {
                return Err(bincode::error::DecodeError::Other(
                    "Invalid DateTime".into(),
                ))
            }
        };
        Ok(TemporaryLog {
            date_time,
            status_code: i32::decode(decoder)?,
            peer: (String::decode(decoder)?, String::decode(decoder)?),
            conn_id: String::decode(decoder)?,
            conn_type: String::decode(decoder)?,
            conn_req: i8::decode(decoder)?,
            conn_res: i8::decode(decoder)?,
            bytes_in: i32::decode(decoder)?,
            bytes_out: i32::decode(decoder)?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogCaptureTimeframe {
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub value: i32, // Now: req_count - res_count (failed/unmatched requests)
    pub high: i32,  // Now: res_count
    pub low: i32,   // Now: req_count
}

impl Clone for TemporaryLog {
    fn clone(&self) -> Self {
        Self {
            date_time: self.date_time,
            status_code: self.status_code,
            peer: self.peer.clone(),
            conn_id: self.conn_id.clone(),
            conn_type: self.conn_type.clone(),
            conn_req: self.conn_req,
            conn_res: self.conn_res,
            bytes_in: self.bytes_in,
            bytes_out: self.bytes_out,
        }
    }
}

#[derive(Debug)]
struct ActiveSegment {
    file_path: PathBuf,
    start_time: DateTime<Utc>,
    mmap_ptr: *mut u8,
    file_descriptor: i32,
    write_offset: usize,
    size: usize,
    logs: VecDeque<TemporaryLog>, // In-memory cache of logs in this segment
}

#[derive(Debug)]
struct ArchivedSegment {
    file_path: PathBuf, // Path to the file as found on disk (could be .bin or .lzma initially)
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

struct LogStore {
    owner: String,
    current_logs: VecDeque<TemporaryLog>, // Global in-memory cache (up to retention)
    active_segment: Option<ActiveSegment>,
    archived_segments: BTreeMap<DateTime<Utc>, ArchivedSegment>,
    base_dir: PathBuf,
    last_rotation_check: DateTime<Utc>,
    segment_duration: Duration,
    retention_period: Duration,
}

const SEGMENT_SIZE: usize = 100 * 1024 * 1024;

static mut PROXY_LOG_STORE: Option<LogStore> = None;
static mut GATEWAY_LOG_STORE: Option<LogStore> = None;

impl LogStore {
    #[allow(deprecated)]
    fn new(owner: String) -> Self {
        let base_dir = PathBuf::from("/tmp/gwrs/logment");
        let _ = fs::create_dir_all(&base_dir);

        let mut store = Self {
            owner: owner.clone(),
            current_logs: VecDeque::new(),
            active_segment: None,
            archived_segments: BTreeMap::new(),
            base_dir: base_dir.clone(),
            last_rotation_check: Utc::now(),
            segment_duration: Duration::minutes(1),
            retention_period: Duration::minutes(35),
        };

        if let Ok(entries) = fs::read_dir(&base_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name.starts_with(&format!("active_segment_{}", owner))
                        && file_name.ends_with(".bin")
                    {
                        log::info!("Found active segment file: {}", file_name);
                    } else if file_name.starts_with(&format!("segment_{}", owner))
                        && (file_name.ends_with(".bin") || file_name.ends_with(".lzma"))
                    {
                        let parts: Vec<&str> = file_name.split('_').collect();
                        if parts.len() == 5 {
                            let _parsed_segment_literal = parts[0];
                            let parsed_owner = parts[1];
                            let date_part_of_start_str = parts[2];
                            let time_part_of_start_str = parts[3];
                            let end_time_of_day_with_ext_str = parts[4];

                            if parsed_owner != owner {
                                continue;
                            }

                            let start_datetime_combined_str =
                                format!("{}_{}", date_part_of_start_str, time_part_of_start_str);
                            let end_time_of_day_str =
                                end_time_of_day_with_ext_str.split('.').next().unwrap_or("");

                            if let Ok(start_time) =
                                Utc.datetime_from_str(&start_datetime_combined_str, "%Y%m%d_%H%M%S")
                            {
                                if let Ok(end_time) = Utc.datetime_from_str(
                                    &format!("{} {}", date_part_of_start_str, end_time_of_day_str),
                                    "%Y%m%d %H%M%S",
                                ) {
                                    store.archived_segments.insert(
                                        start_time,
                                        ArchivedSegment {
                                            file_path: path.clone(),
                                            start_time,
                                            end_time,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        store
    }

    fn ensure_active_segment(&mut self) -> Result<(), LogStoreError> {
        if self.active_segment.is_some() {
            return Ok(());
        }

        let now_for_new_segment = Utc::now();

        let mut existing_active_path: Option<PathBuf> = None;
        let mut existing_segment_parsed_start_time: Option<DateTime<Utc>> = None;

        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name.starts_with(&format!("active_segment_{}", self.owner))
                        && file_name.ends_with(".bin")
                    {
                        let parts: Vec<&str> = file_name.split('_').collect();
                        if parts.len() == 4 {
                            if let Some(ts_str) = parts.last().and_then(|p| p.strip_suffix(".bin"))
                            {
                                if let Ok(ts) = ts_str.parse::<i64>() {
                                    if let chrono::LocalResult::Single(dt) =
                                        Utc.timestamp_opt(ts, 0)
                                    {
                                        existing_active_path = Some(path.clone());
                                        existing_segment_parsed_start_time = Some(dt);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let (segment_file_path, segment_start_time) = if let Some(p) = existing_active_path {
            (p, existing_segment_parsed_start_time.unwrap())
        } else {
            let new_start_time = now_for_new_segment;
            let new_file_name = format!(
                "active_segment_{}_{}.bin",
                self.owner,
                new_start_time.timestamp()
            );
            let new_path = self.base_dir.join(new_file_name);
            (new_path, new_start_time)
        };

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&segment_file_path)?;
        let metadata = file.metadata()?;
        let on_disk_size_before_resize = metadata.len() as usize;

        if on_disk_size_before_resize < SEGMENT_SIZE {
            file.set_len(SEGMENT_SIZE as u64)?;
        }

        let fd = file.as_raw_fd();
        let mmap_ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                SEGMENT_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if mmap_ptr == libc::MAP_FAILED {
            let err = io::Error::last_os_error();
            return Err(LogStoreError::IoError(err));
        }

        let mut loaded_logs_from_disk = VecDeque::new();
        let mut current_write_offset = 0;

        if existing_segment_parsed_start_time.is_some() && on_disk_size_before_resize > 0 {
            let readable_size = on_disk_size_before_resize.min(SEGMENT_SIZE);
            let content_slice =
                unsafe { std::slice::from_raw_parts(mmap_ptr as *const u8, readable_size) };

            let mut offset = 0;
            while offset + 4 <= readable_size {
                let size_bytes = &content_slice[offset..offset + 4];
                let entry_size = u32::from_ne_bytes([
                    size_bytes[0],
                    size_bytes[1],
                    size_bytes[2],
                    size_bytes[3],
                ]) as usize;
                offset += 4;

                if entry_size == 0 {
                    offset -= 4;
                    break;
                }
                if offset + entry_size > readable_size {
                    offset -= 4;
                    break;
                }

                let entry_data = &content_slice[offset..offset + entry_size];
                match bincode::decode_from_slice::<TemporaryLog, _>(
                    entry_data,
                    bincode::config::standard(),
                ) {
                    Ok((log, _)) => loaded_logs_from_disk.push_back(log),
                    Err(e) => {
                        log::error!("Error decoding log entry from active segment: {}", e);
                        break;
                    }
                }
                offset += entry_size;
            }
            current_write_offset = offset;
        }

        self.active_segment = Some(ActiveSegment {
            file_path: segment_file_path,
            start_time: segment_start_time,
            mmap_ptr: mmap_ptr as *mut u8,
            file_descriptor: fd,
            write_offset: current_write_offset,
            size: SEGMENT_SIZE,
            logs: loaded_logs_from_disk,
        });
        Ok(())
    }

    fn check_segment_rotation(&mut self) -> Result<(), LogStoreError> {
        let now = Utc::now();
        if (now - self.last_rotation_check).num_seconds() < 10 {
            return Ok(());
        }
        self.last_rotation_check = now;

        if let Some(active) = &self.active_segment {
            if now.signed_duration_since(active.start_time) >= self.segment_duration {
                self.rotate_segment(now)?;
            }
        }
        Ok(())
    }

    fn rotate_segment(&mut self, rotation_time: DateTime<Utc>) -> Result<(), LogStoreError> {
        if let Some(segment_to_archive) = self.active_segment.take() {
            unsafe {
                if libc::msync(
                    segment_to_archive.mmap_ptr as *mut libc::c_void,
                    segment_to_archive.write_offset,
                    libc::MS_SYNC,
                ) == -1
                {}
            }

            let original_file_path = segment_to_archive.file_path.clone();
            let final_write_offset = segment_to_archive.write_offset;

            unsafe {
                if libc::munmap(
                    segment_to_archive.mmap_ptr as *mut libc::c_void,
                    segment_to_archive.size,
                ) == -1
                {}
                if libc::close(segment_to_archive.file_descriptor) == -1 {}
            }

            if final_write_offset < SEGMENT_SIZE {
                let _ = OpenOptions::new()
                    .write(true)
                    .open(&original_file_path)
                    .and_then(|file_to_truncate| {
                        file_to_truncate.set_len(final_write_offset as u64)
                    });
            }

            let archived_file_name = format!(
                "segment_{}_{}_{}.bin",
                self.owner,
                segment_to_archive.start_time.format("%Y%m%d_%H%M%S"),
                rotation_time.format("%H%M%S")
            );
            let final_archived_file_path = self.base_dir.join(&archived_file_name);

            if let Err(e) = fs::rename(&original_file_path, &final_archived_file_path) {
                self.ensure_active_segment()?;
                return Err(LogStoreError::IoError(e));
            }

            self.archived_segments.insert(
                segment_to_archive.start_time,
                ArchivedSegment {
                    file_path: final_archived_file_path.clone(),
                    start_time: segment_to_archive.start_time,
                    end_time: rotation_time,
                },
            );

            std::thread::spawn(move || {
                let mut input_file_data = Vec::new();
                match File::open(&final_archived_file_path) {
                    Ok(mut file_handle) => {
                        if file_handle.read_to_end(&mut input_file_data).is_ok() {
                            if input_file_data.is_empty() {
                                if fs::remove_file(&final_archived_file_path).is_err() {}
                                return;
                            }

                            let path_for_compressed_file =
                                final_archived_file_path.with_extension("lzma");
                            match File::create(&path_for_compressed_file) {
                                Ok(compressed_file_handle) => {
                                    let mut buffered_writer =
                                        io::BufWriter::new(compressed_file_handle);
                                    if lzma_rs::lzma_compress(
                                        &mut io::Cursor::new(&input_file_data),
                                        &mut buffered_writer,
                                    )
                                    .is_ok()
                                        && buffered_writer.flush().is_ok()
                                    {
                                        if fs::remove_file(&final_archived_file_path).is_err() {}
                                    } else {
                                        let _ = fs::remove_file(&path_for_compressed_file);
                                    }
                                }
                                Err(e) => {
                                    log::error!(
                                        "Error creating compressed file {}: {}",
                                        path_for_compressed_file.display(),
                                        e
                                    );
                                }
                            }
                        } else {
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Error opening file {}: {}",
                            final_archived_file_path.display(),
                            e
                        );
                    }
                }
            });
            self.prune_old_segments(rotation_time)?;
        }
        self.active_segment = None;
        self.ensure_active_segment()?;
        Ok(())
    }

    fn prune_old_segments(&mut self, now: DateTime<Utc>) -> Result<(), LogStoreError> {
        let cutoff_time = now - self.retention_period;

        let keys_of_segments_to_remove: Vec<_> = self
            .archived_segments
            .iter()
            .filter(|(_, segment_metadata)| segment_metadata.end_time < cutoff_time)
            .map(|(key, _)| *key)
            .collect();

        for segment_key in keys_of_segments_to_remove {
            if let Some(segment_to_delete) = self.archived_segments.remove(&segment_key) {
                let path_bin = segment_to_delete.file_path.with_extension("bin");
                let path_lzma = segment_to_delete.file_path.with_extension("lzma");

                if path_lzma.exists() {
                    if let Err(e) = fs::remove_file(&path_lzma) {
                        log::error!("Error deleting file {}: {}", path_lzma.display(), e);
                    }
                } else if path_bin.exists() {
                    if let Err(e) = fs::remove_file(&path_bin) {
                        log::error!("Error deleting file {}: {}", path_bin.display(), e);
                    }
                }
            }
        }

        while let Some(log_entry_in_memory) = self.current_logs.front() {
            if log_entry_in_memory.date_time < cutoff_time {
                self.current_logs.pop_front();
            } else {
                break;
            }
        }
        Ok(())
    }

    // MODIFIED: Added logging before serialization
    fn append_data(&mut self, log: TemporaryLog) -> Result<(), LogStoreError> {
        self.check_segment_rotation()?;
        self.ensure_active_segment()?;

        let active_seg_ref = self.active_segment.as_mut().ok_or_else(|| {
            LogStoreError::IoError(io::Error::new(
                io::ErrorKind::Other,
                "Active segment unexpectedly None after ensure",
            ))
        })?;

        let serialized_log_buffer = bincode::encode_to_vec(&log, bincode::config::standard())
            .map_err(|e| LogStoreError::SerializationError(e.to_string()))?;
        let log_entry_size = serialized_log_buffer.len();
        let total_space_needed_for_entry = log_entry_size + std::mem::size_of::<u32>();

        if active_seg_ref.write_offset + total_space_needed_for_entry > active_seg_ref.size {
            self.rotate_segment(Utc::now())?;

            let new_active_seg_ref = self.active_segment.as_mut().ok_or_else(|| {
                LogStoreError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    "New active segment not available immediately after rotation",
                ))
            })?;

            if new_active_seg_ref.write_offset + total_space_needed_for_entry
                > new_active_seg_ref.size
            {
                return Err(LogStoreError::IoError(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "Log entry too large even for a new segment",
                )));
            }
            unsafe {
                let mmap_write_ptr = new_active_seg_ref
                    .mmap_ptr
                    .add(new_active_seg_ref.write_offset);
                ptr::copy_nonoverlapping(
                    (log_entry_size as u32).to_ne_bytes().as_ptr(),
                    mmap_write_ptr,
                    4,
                );
                ptr::copy_nonoverlapping(
                    serialized_log_buffer.as_ptr(),
                    mmap_write_ptr.add(4),
                    log_entry_size,
                );
            }
            new_active_seg_ref.write_offset += total_space_needed_for_entry;
            new_active_seg_ref.logs.push_back(log.clone());
            self.current_logs.push_back(log);
        } else {
            unsafe {
                let mmap_write_ptr = active_seg_ref.mmap_ptr.add(active_seg_ref.write_offset);
                ptr::copy_nonoverlapping(
                    (log_entry_size as u32).to_ne_bytes().as_ptr(),
                    mmap_write_ptr,
                    4,
                );
                ptr::copy_nonoverlapping(
                    serialized_log_buffer.as_ptr(),
                    mmap_write_ptr.add(4),
                    log_entry_size,
                );
            }
            active_seg_ref.write_offset += total_space_needed_for_entry;
            active_seg_ref.logs.push_back(log.clone());
            self.current_logs.push_back(log);
        }

        if let Some(oldest_log_entry) = self.current_logs.front() {
            if Utc::now().signed_duration_since(oldest_log_entry.date_time) > self.retention_period
            {
                let cutoff_datetime_for_memory = Utc::now() - self.retention_period;
                while self.current_logs.front().map_or(false, |log_to_check| {
                    log_to_check.date_time < cutoff_datetime_for_memory
                }) {
                    self.current_logs.pop_front();
                }
            }
        }
        Ok(())
    }

    fn load_logs(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TemporaryLog>, LogStoreError> {
        let mut result_logs_vec = Vec::new();
        let mut unique_log_keys_set: HashSet<(String, i64, u32)> = HashSet::new();

        let add_if_in_range =
            |log: TemporaryLog,
             _source_name: &str,
             logs_container: &mut Vec<TemporaryLog>,
             _keys_container: &mut HashSet<(String, i64, u32)>| {
                if log.date_time >= start && log.date_time <= end {
                    logs_container.push(log);
                }
            };

        for log_entry in &self.current_logs {
            add_if_in_range(
                log_entry.clone(),
                "current_logs_cache",
                &mut result_logs_vec,
                &mut unique_log_keys_set,
            );
        }

        if let Some(active_seg) = &self.active_segment {
            for log_entry in &active_seg.logs {
                add_if_in_range(
                    log_entry.clone(),
                    "active_segment_memory_cache",
                    &mut result_logs_vec,
                    &mut unique_log_keys_set,
                );
            }

            if active_seg.file_path.exists() && active_seg.write_offset > 0 {
                let active_file_content_slice = unsafe {
                    std::slice::from_raw_parts(
                        active_seg.mmap_ptr as *const u8,
                        active_seg.write_offset,
                    )
                };
                let mut offset = 0;
                while offset + 4 <= active_seg.write_offset {
                    let size_bytes = &active_file_content_slice[offset..offset + 4];
                    let entry_size = u32::from_ne_bytes([
                        size_bytes[0],
                        size_bytes[1],
                        size_bytes[2],
                        size_bytes[3],
                    ]) as usize;
                    offset += 4;

                    if entry_size == 0 || offset + entry_size > active_seg.write_offset {
                        break;
                    }
                    let entry_data = &active_file_content_slice[offset..offset + entry_size];
                    match bincode::decode_from_slice::<TemporaryLog, _>(
                        entry_data,
                        bincode::config::standard(),
                    ) {
                        Ok((log_disk_entry, _)) => {
                            add_if_in_range(
                                log_disk_entry,
                                "active_segment_disk_file",
                                &mut result_logs_vec,
                                &mut unique_log_keys_set,
                            );
                        }
                        Err(e) => {
                            log::error!("Error decoding log entry from active segment: {}", e);
                            break;
                        }
                    }
                    offset += entry_size;
                }
            }
        }

        for (_, archived_segment_info) in self.archived_segments.iter() {
            if archived_segment_info.start_time <= end && archived_segment_info.end_time >= start {
                match load_logs_from_segment(archived_segment_info, start, end) {
                    Ok(logs_from_one_archive) => {
                        for log_entry_archived in logs_from_one_archive {
                            add_if_in_range(
                                log_entry_archived,
                                "archived_segment_file",
                                &mut result_logs_vec,
                                &mut unique_log_keys_set,
                            );
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Error loading logs from archived segment {}: {}",
                            archived_segment_info.file_path.display(),
                            e
                        );
                    }
                }
            }
        }

        result_logs_vec.sort_by(|a, b| a.date_time.cmp(&b.date_time));

        Ok(result_logs_vec)
    }

    // MODIFIED: get_data_time_frame with enhanced logging
    fn get_data_time_frame(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let logs = self.load_logs(start, end)?;
        // Your existing log::error!("Data: {:#?}", logs); // This is where you see the issue

        let mut result = Vec::new();
        let start_ts_interval = start.timestamp() / 15;
        let end_ts_interval = end.timestamp() / 15;

        if logs.is_empty() {
            for interval_block_ts in start_ts_interval..=end_ts_interval {
                let interval_datetime = Utc
                    .timestamp_opt(interval_block_ts * 15, 0)
                    .single()
                    .unwrap_or_else(|| {
                        Utc.timestamp_opt(start.timestamp(), 0)
                            .single()
                            .unwrap_or(start)
                    });
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
            return Ok(result);
        }

        let mut time_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();
        for log in logs.iter() {
            let interval_ts = log.date_time.timestamp() / 15;
            time_groups.entry(interval_ts).or_default().push(log);
        }

        let mut interval_results = HashMap::new();
        if time_groups.is_empty() && !logs.is_empty() {}

        for (interval_block_ts, interval_logs_in_group) in time_groups {
            let interval_datetime = Utc
                .timestamp_opt(interval_block_ts * 15, 0)
                .single()
                .unwrap_or_else(|| {
                    Utc.timestamp_opt(start.timestamp(), 0)
                        .single()
                        .unwrap_or(start)
                });

            let mut req_count = 0_i32;
            let mut res_count = 0_i32;

            if !interval_logs_in_group.is_empty() {
                for (_idx, log_entry) in interval_logs_in_group.iter().enumerate() {
                    // Using conn_req and conn_res directly as per your updated logic
                    req_count += log_entry.conn_req as i32;
                    res_count += log_entry.conn_res as i32;
                }
            }

            let failed_or_unmatched_count = req_count - res_count;

            interval_results.insert(
                interval_block_ts,
                LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: failed_or_unmatched_count,
                    high: res_count,
                    low: req_count,
                },
            );
        }

        for interval_block_ts in start_ts_interval..=end_ts_interval {
            let timeframe = interval_results
                .get(&interval_block_ts)
                .cloned()
                .unwrap_or_else(|| {
                    let interval_datetime = Utc
                        .timestamp_opt(interval_block_ts * 15, 0)
                        .single()
                        .unwrap_or_else(|| {
                            Utc.timestamp_opt(start.timestamp(), 0)
                                .single()
                                .unwrap_or(start)
                        });
                    LogCaptureTimeframe {
                        date_time: interval_datetime,
                        value: 0,
                        high: 0,
                        low: 0,
                    }
                });
            result.push(timeframe);
        }
        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        Ok(result)
    }

    fn get_data_time_frame_by_status_code(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status_filter: i32,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let logs = self.load_logs(start, end)?;
        let mut result = Vec::new();
        let start_ts_interval = start.timestamp() / 15;
        let end_ts_interval = end.timestamp() / 15;

        if logs.is_empty() {
            for interval_block_ts in start_ts_interval..=end_ts_interval {
                let interval_datetime = Utc
                    .timestamp_opt(interval_block_ts * 15, 0)
                    .single()
                    .unwrap_or(start);
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
            return Ok(result);
        }

        let mut conn_logs_map: HashMap<String, Vec<&TemporaryLog>> = HashMap::new();
        for log_ref in logs.iter() {
            conn_logs_map
                .entry(log_ref.conn_id.clone())
                .or_default()
                .push(log_ref);
        }

        let mut interval_response_times_map: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut interval_direct_status_counts_map: HashMap<i64, i32> = HashMap::new();

        for (_conn_id_key, single_conn_logs_vec) in &conn_logs_map {
            let mut sorted_logs_for_conn = single_conn_logs_vec.clone();
            sorted_logs_for_conn.sort_by_key(|log_item| log_item.date_time);

            let mut req_time_for_conn: Option<DateTime<Utc>> = None;
            for current_log in sorted_logs_for_conn {
                if current_log.conn_type == "REQ" || current_log.conn_type == "DOWNSTREAM" {
                    req_time_for_conn = Some(current_log.date_time);
                } else if (current_log.conn_type == "RES" || current_log.conn_type == "UPSTREAM")
                    && current_log.status_code == status_filter
                {
                    if let Some(rt) = req_time_for_conn.take() {
                        let resp_time_ms = current_log
                            .date_time
                            .signed_duration_since(rt)
                            .num_milliseconds();
                        if resp_time_ms >= 0 {
                            let interval_ts_key = current_log.date_time.timestamp() / 15;
                            interval_response_times_map
                                .entry(interval_ts_key)
                                .or_default()
                                .push(resp_time_ms);
                        }
                    }
                }
                if current_log.status_code == status_filter {
                    let interval_ts_key = current_log.date_time.timestamp() / 15;
                    *interval_direct_status_counts_map
                        .entry(interval_ts_key)
                        .or_default() += 1;
                }
            }
        }

        let mut final_interval_results_map = HashMap::new();
        for interval_block_ts_key in start_ts_interval..=end_ts_interval {
            let interval_dt = Utc
                .timestamp_opt(interval_block_ts_key * 15, 0)
                .single()
                .unwrap_or(start);
            if let Some(resp_times_vec) = interval_response_times_map.get(&interval_block_ts_key) {
                if !resp_times_vec.is_empty() {
                    let count_val = resp_times_vec.len() as i32;
                    let max_rt = resp_times_vec.iter().max().map_or(0, |&t_val| t_val as i32);
                    let min_rt = resp_times_vec.iter().min().map_or(0, |&t_val| t_val as i32);
                    final_interval_results_map.insert(
                        interval_block_ts_key,
                        LogCaptureTimeframe {
                            date_time: interval_dt,
                            value: count_val,
                            high: max_rt,
                            low: min_rt,
                        },
                    );
                    continue;
                }
            }
            let direct_count = interval_direct_status_counts_map
                .get(&interval_block_ts_key)
                .copied()
                .unwrap_or(0);
            final_interval_results_map.insert(
                interval_block_ts_key,
                LogCaptureTimeframe {
                    date_time: interval_dt,
                    value: direct_count,
                    high: 0,
                    low: 0,
                },
            );
        }

        for interval_block_ts_key in start_ts_interval..=end_ts_interval {
            result.push(
                final_interval_results_map
                    .remove(&interval_block_ts_key)
                    .unwrap_or_else(|| {
                        let interval_dt_fallback = Utc
                            .timestamp_opt(interval_block_ts_key * 15, 0)
                            .single()
                            .unwrap_or(start);
                        LogCaptureTimeframe {
                            date_time: interval_dt_fallback,
                            value: 0,
                            high: 0,
                            low: 0,
                        }
                    }),
            );
        }
        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        Ok(result)
    }

    fn get_data_time_frame_by_conn_stall(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let logs = self.load_logs(start, end)?;
        let mut result = Vec::new();
        let start_ts_interval = start.timestamp() / 15;
        let end_ts_interval = end.timestamp() / 15;

        if logs.is_empty() {
            for interval_block_ts in start_ts_interval..=end_ts_interval {
                let interval_datetime = Utc
                    .timestamp_opt(interval_block_ts * 15, 0)
                    .single()
                    .unwrap_or(start);
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
            return Ok(result);
        }

        let mut interval_stalled_log_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();
        for log_item in logs.iter().filter(|l| l.conn_req == 1 && l.conn_res == 0) {
            let interval_ts_key = log_item.date_time.timestamp() / 15;
            interval_stalled_log_groups
                .entry(interval_ts_key)
                .or_default()
                .push(log_item);
        }

        let mut final_interval_results_map = HashMap::new();
        for (interval_block_ts_key, logs_in_interval) in interval_stalled_log_groups {
            let interval_dt = Utc
                .timestamp_opt(interval_block_ts_key * 15, 0)
                .single()
                .unwrap_or(start);

            let mut distinct_stalled_conn_ids = std::collections::HashSet::new();
            let mut earliest_stall_ts_in_interval = i64::MAX;
            let mut latest_stall_ts_in_interval = 0i64;

            if !logs_in_interval.is_empty() {
                earliest_stall_ts_in_interval = logs_in_interval
                    .iter()
                    .map(|l| l.date_time.timestamp())
                    .min()
                    .unwrap_or(i64::MAX);
                latest_stall_ts_in_interval = logs_in_interval
                    .iter()
                    .map(|l| l.date_time.timestamp())
                    .max()
                    .unwrap_or(0);
                for log_entry in &logs_in_interval {
                    distinct_stalled_conn_ids.insert(log_entry.conn_id.as_str());
                }
            }
            let distinct_stalls_count = distinct_stalled_conn_ids.len() as i32;

            final_interval_results_map.insert(
                interval_block_ts_key,
                LogCaptureTimeframe {
                    date_time: interval_dt,
                    value: distinct_stalls_count,
                    high: if distinct_stalls_count > 0 {
                        latest_stall_ts_in_interval as i32
                    } else {
                        0
                    },
                    low: if distinct_stalls_count > 0 {
                        earliest_stall_ts_in_interval as i32
                    } else {
                        0
                    },
                },
            );
        }

        for interval_block_ts_key in start_ts_interval..=end_ts_interval {
            result.push(
                final_interval_results_map
                    .remove(&interval_block_ts_key)
                    .unwrap_or_else(|| {
                        let interval_dt_fallback = Utc
                            .timestamp_opt(interval_block_ts_key * 15, 0)
                            .single()
                            .unwrap_or(start);
                        LogCaptureTimeframe {
                            date_time: interval_dt_fallback,
                            value: 0,
                            high: 0,
                            low: 0,
                        }
                    }),
            );
        }
        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        Ok(result)
    }

    fn get_bytes_io_frame(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        metric: BytesMetric,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let logs = self.load_logs(start, end)?;
        let mut result = Vec::new();
        let start_ts_interval = start.timestamp() / 15;
        let end_ts_interval = end.timestamp() / 15;

        if logs.is_empty() {
            for interval_block_ts in start_ts_interval..=end_ts_interval {
                let interval_datetime = Utc
                    .timestamp_opt(interval_block_ts * 15, 0)
                    .single()
                    .unwrap_or(start);
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
            return Ok(result);
        }

        let mut interval_log_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();
        for log_item in logs.iter() {
            let interval_ts_key = log_item.date_time.timestamp() / 15;
            interval_log_groups
                .entry(interval_ts_key)
                .or_default()
                .push(log_item);
        }

        let mut final_interval_results_map = HashMap::new();
        let get_bytes_value_for_log = |log_entry: &&TemporaryLog| -> i32 {
            match metric {
                BytesMetric::BytesIn => log_entry.bytes_in,
                BytesMetric::BytesOut => log_entry.bytes_out,
                BytesMetric::BytesTotal => log_entry.bytes_in + log_entry.bytes_out,
            }
        };

        for (interval_block_ts_key, logs_in_interval) in interval_log_groups {
            let interval_dt = Utc
                .timestamp_opt(interval_block_ts_key * 15, 0)
                .single()
                .unwrap_or(start);

            let sum_bytes_in_interval: i64 = logs_in_interval
                .iter()
                .map(|l_val| get_bytes_value_for_log(l_val) as i64)
                .sum();
            let avg_bytes_for_15s_interval = if !logs_in_interval.is_empty() {
                (sum_bytes_in_interval / logs_in_interval.len() as i64) as i32
            } else {
                0
            };

            let mut per_second_byte_values_map: HashMap<i64, Vec<i32>> = HashMap::new();
            for log_entry_in_interval in &logs_in_interval {
                let second_ts_key = log_entry_in_interval.date_time.timestamp();
                per_second_byte_values_map
                    .entry(second_ts_key)
                    .or_default()
                    .push(get_bytes_value_for_log(log_entry_in_interval));
            }

            let mut per_second_avg_bytes_vec: Vec<i32> = Vec::new();
            for (_sec_ts_key, byte_values_for_second) in per_second_byte_values_map {
                if !byte_values_for_second.is_empty() {
                    let sum_bytes_for_second: i64 = byte_values_for_second
                        .iter()
                        .map(|&b_val| b_val as i64)
                        .sum();
                    per_second_avg_bytes_vec
                        .push((sum_bytes_for_second / byte_values_for_second.len() as i64) as i32);
                }
            }
            let highest_one_sec_avg = per_second_avg_bytes_vec.iter().max().copied().unwrap_or(0);
            let lowest_one_sec_avg = per_second_avg_bytes_vec.iter().min().copied().unwrap_or(0);

            final_interval_results_map.insert(
                interval_block_ts_key,
                LogCaptureTimeframe {
                    date_time: interval_dt,
                    value: avg_bytes_for_15s_interval,
                    high: highest_one_sec_avg,
                    low: lowest_one_sec_avg,
                },
            );
        }

        for interval_block_ts_key in start_ts_interval..=end_ts_interval {
            result.push(
                final_interval_results_map
                    .remove(&interval_block_ts_key)
                    .unwrap_or_else(|| {
                        let interval_dt_fallback = Utc
                            .timestamp_opt(interval_block_ts_key * 15, 0)
                            .single()
                            .unwrap_or(start);
                        LogCaptureTimeframe {
                            date_time: interval_dt_fallback,
                            value: 0,
                            high: 0,
                            low: 0,
                        }
                    }),
            );
        }
        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        Ok(result)
    }
}

fn load_logs_from_segment(
    segment_info: &ArchivedSegment,
    query_start_time: DateTime<Utc>,
    query_end_time: DateTime<Utc>,
) -> Result<Vec<TemporaryLog>, LogStoreError> {
    let mut loaded_logs_vec = Vec::new();
    if segment_info.end_time < query_start_time || segment_info.start_time > query_end_time {
        return Ok(loaded_logs_vec);
    }

    let path_compressed = segment_info.file_path.with_extension("lzma");
    let path_uncompressed =
        if segment_info.file_path.extension().and_then(|e| e.to_str()) == Some("lzma") {
            segment_info.file_path.with_extension("bin")
        } else {
            segment_info.file_path.clone()
        };

    let (path_to_load, is_compressed) = if path_compressed.exists() {
        (path_compressed, true)
    } else if path_uncompressed.exists() {
        (path_uncompressed, false)
    } else {
        return Ok(loaded_logs_vec);
    };

    let mut file_bytes = Vec::new();
    File::open(&path_to_load)?.read_to_end(&mut file_bytes)?;

    let data_to_process: &[u8];
    let mut decompressed_data_holder;

    if is_compressed {
        decompressed_data_holder = Vec::new();
        lzma_rs::lzma_decompress(
            &mut io::Cursor::new(&file_bytes),
            &mut decompressed_data_holder,
        )
        .map_err(|e| {
            LogStoreError::CompressionError(format!(
                "LZMA decompression for {:?}: {:?}",
                path_to_load, e
            ))
        })?;
        data_to_process = &decompressed_data_holder;
    } else {
        data_to_process = &file_bytes;
    }

    let data_len = data_to_process.len();
    let mut offset = 0;
    while offset + 4 <= data_len {
        let size_bytes = &data_to_process[offset..offset + 4];
        let entry_size =
            u32::from_ne_bytes([size_bytes[0], size_bytes[1], size_bytes[2], size_bytes[3]])
                as usize;
        offset += 4;
        if entry_size == 0 || offset + entry_size > data_len {
            break;
        }
        let entry_data = &data_to_process[offset..offset + entry_size];
        match bincode::decode_from_slice::<TemporaryLog, _>(entry_data, bincode::config::standard())
        {
            Ok((log, _)) => {
                if log.date_time >= query_start_time && log.date_time <= query_end_time {
                    loaded_logs_vec.push(log);
                }
            }
            Err(_e) => {
                break;
            }
        }
        offset += entry_size;
    }
    Ok(loaded_logs_vec)
}

#[allow(static_mut_refs)]
pub fn init() {
    unsafe {
        if PROXY_LOG_STORE.is_none() {
            PROXY_LOG_STORE = Some(LogStore::new("proxy".to_string()));
        }
        if GATEWAY_LOG_STORE.is_none() {
            GATEWAY_LOG_STORE = Some(LogStore::new("gateway".to_string()));
        }
    }
}

#[allow(static_mut_refs, dead_code)]
pub mod tlog_proxy {
    use super::*;
    pub fn append_data(log: TemporaryLog) -> Result<(), LogStoreError> {
        unsafe {
            if PROXY_LOG_STORE.is_none() {
                init();
            }
            PROXY_LOG_STORE
                .as_mut()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Proxy log store not initialized",
                    ))
                })?
                .append_data(log)
        }
    }
    pub fn get_data_time_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if PROXY_LOG_STORE.is_none() {
                init();
            }
            PROXY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Proxy log store not initialized",
                    ))
                })?
                .get_data_time_frame(start, end)
        }
    }
    pub fn get_data_time_frame_by_status_code(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status_filter: i32,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if PROXY_LOG_STORE.is_none() {
                init();
            }
            PROXY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Proxy log store not initialized",
                    ))
                })?
                .get_data_time_frame_by_status_code(start, end, status_filter)
        }
    }
    pub fn get_data_time_frame_by_conn_stall(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if PROXY_LOG_STORE.is_none() {
                init();
            }
            PROXY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Proxy log store not initialized",
                    ))
                })?
                .get_data_time_frame_by_conn_stall(start, end)
        }
    }
    pub fn get_bytes_io_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        metric: BytesMetric,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if PROXY_LOG_STORE.is_none() {
                init();
            }
            PROXY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Proxy log store not initialized",
                    ))
                })?
                .get_bytes_io_frame(start, end, metric)
        }
    }
}

#[allow(static_mut_refs, dead_code)]
pub mod tlog_gateway {
    use super::*;
    pub fn append_data(log: TemporaryLog) -> Result<(), LogStoreError> {
        unsafe {
            if GATEWAY_LOG_STORE.is_none() {
                init();
            }
            GATEWAY_LOG_STORE
                .as_mut()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Gateway log store not initialized",
                    ))
                })?
                .append_data(log)
        }
    }
    pub fn get_data_time_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if GATEWAY_LOG_STORE.is_none() {
                init();
            }
            GATEWAY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Gateway log store not initialized",
                    ))
                })?
                .get_data_time_frame(start, end)
        }
    }
    pub fn get_data_time_frame_by_status_code(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status_filter: i32,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if GATEWAY_LOG_STORE.is_none() {
                init();
            }
            GATEWAY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Gateway log store not initialized",
                    ))
                })?
                .get_data_time_frame_by_status_code(start, end, status_filter)
        }
    }
    pub fn get_data_time_frame_by_conn_stall(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if GATEWAY_LOG_STORE.is_none() {
                init();
            }
            GATEWAY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Gateway log store not initialized",
                    ))
                })?
                .get_data_time_frame_by_conn_stall(start, end)
        }
    }
    pub fn get_bytes_io_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        metric: BytesMetric,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        unsafe {
            if GATEWAY_LOG_STORE.is_none() {
                init();
            }
            GATEWAY_LOG_STORE
                .as_ref()
                .ok_or_else(|| {
                    LogStoreError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        "Gateway log store not initialized",
                    ))
                })?
                .get_bytes_io_frame(start, end, metric)
        }
    }
}
