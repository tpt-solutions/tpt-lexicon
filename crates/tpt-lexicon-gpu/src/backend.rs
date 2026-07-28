//! GPU backend definitions and stub implementations.

use alloc::vec::Vec;

/// Available GPU backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuBackend {
    /// Portable wgpu compute shader backend.
    Wgpu,
    /// Raw CUDA backend (feature-gated).
    Cuda,
    /// Raw Metal backend (feature-gated).
    Metal,
}

impl GpuBackend {
    /// Returns the name of the backend.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wgpu => "wgpu",
            Self::Cuda => "cuda",
            Self::Metal => "metal",
        }
    }

    /// Returns `true` if the backend is available on this platform.
    pub fn is_available(&self) -> bool {
        match self {
            Self::Wgpu => cfg!(feature = "wgpu"),
            Self::Cuda => cfg!(feature = "cuda"),
            Self::Metal => cfg!(feature = "metal"),
        }
    }
}

/// A GPU-accelerated tokenizer.
///
/// Wraps a backend and provides parallel tokenization over GPU shared memory.
///
/// # Status
///
/// This is a stub implementation. The actual GPU compute shaders and
/// buffer management are not yet implemented. All methods panic with
/// `unimplemented!()` if the backend is not available.
#[derive(Debug)]
pub struct GpuTokenizer {
    backend: GpuBackend,
}

impl GpuTokenizer {
    /// Create a new GPU tokenizer with the given backend.
    ///
    /// # Panics
    ///
    /// Panics if the requested backend is not available.
    pub fn new(backend: GpuBackend) -> Self {
        if !backend.is_available() {
            panic!(
                "GPU backend '{}' is not available. Enable the corresponding feature.",
                backend.name()
            );
        }
        Self { backend }
    }

    /// Returns the backend type.
    #[inline]
    pub fn backend(&self) -> GpuBackend {
        self.backend
    }

    /// Tokenize input bytes on the GPU.
    ///
    /// # Panics
    ///
    /// Currently unimplemented — this is a stub for the GPU pipeline.
    pub fn tokenize(&self, _input: &[u8]) -> Vec<u32> {
        unimplemented!("GPU tokenization is not yet implemented")
    }

    /// Returns the maximum input size in bytes that the GPU buffer can hold.
    pub fn max_input_size(&self) -> usize {
        unimplemented!("GPU buffer sizing is not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VERSION;

    #[test]
    fn backend_names() {
        assert_eq!(GpuBackend::Wgpu.name(), "wgpu");
        assert_eq!(GpuBackend::Cuda.name(), "cuda");
        assert_eq!(GpuBackend::Metal.name(), "metal");
    }

    #[test]
    fn backend_availability() {
        // Without features enabled, none should be available.
        assert!(!GpuBackend::Wgpu.is_available());
        assert!(!GpuBackend::Cuda.is_available());
        assert!(!GpuBackend::Metal.is_available());
    }

    #[test]
    #[should_panic(expected = "GPU backend")]
    fn new_panics_when_unavailable() {
        let _ = GpuTokenizer::new(GpuBackend::Wgpu);
    }

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }
}
