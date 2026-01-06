//! Output buffer for audio synthesis

/// Time table for more accurate C64 simulation
const TIME_TABLE: [[u32; 5]; 5] = [
    [162, 167, 167, 127, 128], // formants synth
    [226, 60, 60, 0, 0],       // unvoiced sample 0
    [225, 60, 59, 0, 0],       // unvoiced sample 1
    [200, 0, 0, 54, 55],       // voiced sample 0
    [199, 0, 0, 54, 54],       // voiced sample 1
];

/// Output buffer for audio synthesis
pub struct OutputBuffer {
    buffer: Vec<u8>,
    buffer_pos: u32,
    old_time_table_index: usize,
}

impl OutputBuffer {
    /// Create a new output buffer with the given size
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![0u8; buffer_size],
            buffer_pos: 0,
            old_time_table_index: 0,
        }
    }

    /// Scale by 16 and write five times
    pub fn write(&mut self, index: usize, a: u8) {
        let scaled = ((a & 15) as u8) * 16;
        self.write_ary(index, [scaled, scaled, scaled, scaled, scaled]);
    }

    /// Write the five given values
    pub fn write_ary(&mut self, index: usize, array: [u8; 5]) {
        self.buffer_pos += TIME_TABLE[self.old_time_table_index][index];
        let pos = (self.buffer_pos / 50) as usize;

        // Check for buffer overflow
        if pos > self.buffer.len() {
            // In production, silently clamp
            return;
        }

        self.old_time_table_index = index;

        // Write a little bit in advance
        for k in 0..5 {
            if pos + k < self.buffer.len() {
                self.buffer[pos + k] = array[k];
            }
        }
    }

    /// Get the filled portion of the buffer
    pub fn get(self) -> Vec<u8> {
        let end = (self.buffer_pos / 50) as usize;
        let end = end.min(self.buffer.len());
        self.buffer[..end].to_vec()
    }
}
