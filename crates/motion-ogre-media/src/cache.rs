//! Frame and proxy caching for efficient media playback
//!
//! This module provides LRU-based caching for decoded frames and generated proxies
//! to enable smooth scrubbing and fast playback performance.

use lru::LruCache;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::types::{VideoFrame, AudioFrame};
use crate::error::{Error, Result};
use std::path::Path;

/// Cached frame entry
#[derive(Debug, Clone)]
pub struct CachedFrame {
    /// The frame data
    pub frame: VideoFrame,
    /// When this frame was cached
    pub cached_at: Instant,
    /// Last access time
    pub last_access: Instant,
    /// Number of times accessed
    pub access_count: u64,
}

impl CachedFrame {
    /// Create a new cached frame
    pub fn new(frame: VideoFrame) -> Self {
        let now = Instant::now();
        Self {
            frame,
            cached_at: now,
            last_access: now,
            access_count: 0,
        }
    }

    /// Mark this frame as accessed
    pub fn mark_accessed(&mut self) {
        self.last_access = Instant::now();
        self.access_count += 1;
    }

    /// Get cache priority score
    pub fn priority_score(&self) -> u64 {
        let age_secs = self.cached_at.elapsed().as_secs();
        let recency_bonus = if self.last_access.elapsed().as_secs() < 10 {
            100
        } else {
            0
        };
        self.access_count * 10 + recency_bonus - age_secs
    }
}

/// LRU cache for video frames
pub struct FrameCache {
    inner: Arc<Mutex<LruCache<String, CachedFrame>>>,
    max_size: usize,
    current_size_bytes: Arc<Mutex<usize>>,
}

impl FrameCache {
    /// Create a new frame cache
    pub fn new(max_frames: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(max_frames))),
            max_size: max_frames,
            current_size_bytes: Arc::new(Mutex::new(0)),
        }
    }

    /// Get frame from cache
    pub fn get(&self, key: &str) -> Option<VideoFrame> {
        let mut cache = self.inner.lock().unwrap();
        if let Some(mut cached) = cache.get_mut(key) {
            cached.mark_accessed();
            
            let frame_size = cached.frame.size_in_bytes();
            Some(cached.frame.clone())
        } else {
            None
        }
    }

    /// Put frame in cache
    pub fn put(&self, key: String, frame: VideoFrame) -> Result<()> {
        let mut cache = self.inner.lock().unwrap();
        let frame_size = frame.size_in_bytes();
        
        cache.put(key, CachedFrame::new(frame));
        
        let mut size = self.current_size_bytes.lock().unwrap();
        *size += frame_size;
        
        Ok(())
    }

    /// Check if frame is in cache
    pub fn contains(&self, key: &str) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.contains(key)
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        let mut size = self.current_size_bytes.lock().unwrap();
        cache.clear();
        *size = 0;
    }

    /// Get current cache size in frames
    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.is_empty()
    }

    /// Get current size in bytes
    pub fn size_bytes(&self) -> usize {
        let size = self.current_size_bytes.lock().unwrap();
        *size
    }

    /// Get cache stats
    pub fn stats(&self) -> FrameCacheStats {
        let cache = self.inner.lock().unwrap();
        let size = self.current_size_bytes.lock().unwrap();
        
        FrameCacheStats {
            frame_count: cache.len(),
            size_bytes: *size,
            max_size: self.max_size,
            utilization_percent: (*size as f64 / (self.max_size * 1024 * 1024) as f64 * 100.0) as u8,
        }
    }
}

/// Frame cache statistics
#[derive(Debug, Clone)]
pub struct FrameCacheStats {
    /// Number of cached frames
    pub frame_count: usize,
    /// Total size in bytes
    pub size_bytes: usize,
    /// Maximum size in bytes
    pub max_size: usize,
    /// Cache utilization percentage (0-100)
    pub utilization_percent: u8,
}

/// Proxy cache entry
#[derive(Debug, Clone)]
pub struct ProxyEntry {
    /// Proxy file path
    pub proxy_path: String,
    /// Quality level
    pub quality: crate::types::ProxyQuality,
    /// Original file path
    pub original_path: String,
    /// Created timestamp
    pub created_at: Instant,
    /// File size
    pub file_size: u64,
}

/// Cache for proxy files
pub struct ProxyCache {
    inner: Arc<Mutex<HashMap<String, ProxyEntry>>>,
    max_entries: usize,
}

impl ProxyCache {
    /// Create a new proxy cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            max_entries,
        }
    }

    /// Get proxy for original file
    pub fn get(&self, original_path: &str) -> Option<ProxyEntry> {
        let cache = self.inner.lock().unwrap();
        cache.get(original_path).cloned()
    }

    /// Add proxy to cache
    pub fn put(&self, entry: ProxyEntry) -> Result<()> {
        let mut cache = self.inner.lock().unwrap();
        
        if cache.len() >= self.max_entries {
            self.evict_oldest(&mut cache)?;
        }
        
        cache.insert(entry.original_path.clone(), entry);
        Ok(())
    }

    /// Remove proxy from cache
    pub fn remove(&self, original_path: &str) -> Result<()> {
        let mut cache = self.inner.lock().unwrap();
        cache.remove(original_path);
        Ok(())
    }

    /// Clear expired entries
    pub fn cleanup(&self, max_age: Duration) -> Result<()> {
        let mut cache = self.inner.lock().unwrap();
        let now = Instant::now();
        
        cache.retain(|_, entry| {
            let age = now.duration_since(entry.created_at);
            age < max_age
        });
        
        Ok(())
    }

    /// Evict oldest entry
    fn evict_oldest(&self, cache: &mut std::sync::MutexGuard<HashMap<String, ProxyEntry>>) -> Result<()> {
        // Simple eviction - remove first entry (avoiding complexity)
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
        Ok(())
    }

    /// Get number of cached proxies
    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Cache for audio waveforms
pub struct WaveformCache {
    inner: Arc<Mutex<LruCache<String, Arc<Vec<f32>>>>>,
    max_entries: usize,
}

impl WaveformCache {
    /// Create a new waveform cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(max_entries))),
            max_entries,
        }
    }

    /// Get waveform from cache
    pub fn get(&self, key: &str) -> Option<Arc<Vec<f32>>> {
        let mut cache = self.inner.lock().unwrap();
        cache.get(key).cloned()
    }

    /// Put waveform in cache
    pub fn put(&self, key: String, waveform: Vec<f32>) -> Result<()> {
        let mut cache = self.inner.lock().unwrap();
        cache.put(key, Arc::new(waveform));
        Ok(())
    }

    /// Get cache stats
    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VideoFrame, PixelFormat};
    use std::time::Duration;

    #[test]
    fn test_frame_cache_basic() {
        let cache = FrameCache::new(10);
        
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_frame_cache_put_and_get() {
        let cache = FrameCache::new(10);
        
        let frame = VideoFrame::new(
            1920,
            1080,
            PixelFormat::Yuv420P,
            vec![0u8; 1920 * 1080],
            Duration::from_millis(100),
            None,
            Duration::from_millis(33),
            true,
        );
        
        cache.put("test_key".to_string(), frame.clone()).unwrap();
        
        assert!(cache.contains("test_key"));
        assert_eq!(cache.len(), 1);
        
        let retrieved = cache.get("test_key").unwrap();
        assert_eq!(retrieved.width, frame.width);
        assert_eq!(retrieved.height, frame.height);
    }

    #[test]
    fn test_frame_cache_eviction() {
        let cache = FrameCache::new(2);
        
        for i in 0..3 {
            let frame = VideoFrame::new(
                1920,
                1080,
                PixelFormat::Yuv420P,
                vec![0u8; 1920 * 1080],
                Duration::from_millis(i * 33),
                None,
                Duration::from_millis(33),
                true,
            );
            
            cache.put(format!("key_{}", i), frame).unwrap();
        }
        
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_proxy_cache_basic() {
        let cache = ProxyCache::new(10);
        
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_waveform_cache_basic() {
        let cache = WaveformCache::new(10);
        
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}