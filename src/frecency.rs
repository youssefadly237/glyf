use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Frecency {
    entries: HashMap<u32, u32>,
    path: PathBuf,
    dirty: bool,
}

impl Frecency {
    pub fn load() -> Self {
        let path = Self::path();
        let entries = Self::read(&path);
        Self {
            entries,
            path,
            dirty: false,
        }
    }

    pub fn get(&self, codepoint: u32) -> u32 {
        self.entries.get(&codepoint).copied().unwrap_or(0)
    }

    pub fn record(&mut self, codepoint: u32) {
        *self.entries.entry(codepoint).or_insert(0) += 1;
        self.dirty = true;
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.dirty {
            self.write()?;
            self.dirty = false;
        }
        Ok(())
    }
}

impl Drop for Frecency {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
impl Frecency {
    pub fn empty_for_testing() -> Self {
        Self {
            entries: HashMap::new(),
            path: PathBuf::new(),
            dirty: false,
        }
    }
}

impl Frecency {
    fn path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                if home.is_empty() {
                    return std::env::temp_dir();
                }
                PathBuf::from(home).join(".local/share/glyf")
            });
        let dir = base.join("glyf");
        fs::create_dir_all(&dir).ok();
        dir.join("frecency.bin")
    }

    fn read(path: &Path) -> HashMap<u32, u32> {
        let Ok(bytes) = fs::read(path) else {
            return HashMap::new();
        };
        if bytes.len() % 8 != 0 {
            return HashMap::new();
        }
        bytes
            .chunks_exact(8)
            .map(|c| {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&c[..4]);
                let cp = u32::from_le_bytes(buf);
                buf.copy_from_slice(&c[4..8]);
                let count = u32::from_le_bytes(buf);
                (cp, count)
            })
            .collect()
    }

    fn write(&self) -> io::Result<()> {
        let mut sorted: Vec<(u32, u32)> = self.entries.iter().map(|(&k, &v)| (k, v)).collect();
        sorted.sort_unstable_by_key(|&(cp, _)| cp);
        let mut bytes = Vec::with_capacity(sorted.len() * 8);
        for &(cp, count) in &sorted {
            bytes.extend_from_slice(&cp.to_le_bytes());
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, &bytes)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_frecency_returns_zero() {
        let f = Frecency::empty_for_testing();
        assert_eq!(f.get(0x0041), 0);
    }

    #[test]
    fn record_then_get() {
        let mut f = Frecency::empty_for_testing();
        f.record(0x0041);
        assert_eq!(f.get(0x0041), 1);
    }

    #[test]
    fn record_multiple_times() {
        let mut f = Frecency::empty_for_testing();
        for _ in 0..5 {
            f.record(0x0041);
        }
        assert_eq!(f.get(0x0041), 5);
    }

    #[test]
    fn record_sets_dirty() {
        let mut f = Frecency::empty_for_testing();
        f.record(0x0041);
        assert!(f.dirty);
    }

    #[test]
    fn flush_noop_when_not_dirty() {
        let mut f = Frecency::empty_for_testing();
        assert!(f.flush().is_ok());
    }

    #[test]
    fn record_multiple_codepoints() {
        let mut f = Frecency::empty_for_testing();
        f.record(0x0041);
        f.record(0x1F600);
        assert_eq!(f.get(0x0041), 1);
        assert_eq!(f.get(0x1F600), 1);
        assert_eq!(f.get(0x0042), 0);
    }
}
