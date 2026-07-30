//! wgpu backend for GPU-accelerated tokenization.
//!
//! This module implements the [`super::GpuTokenizer`] logic using wgpu compute
//! shaders. It is only available when the `wgpu` feature is enabled.

#![cfg(feature = "wgpu")]

use alloc::vec::Vec;

use crate::GpuError;

/// GPU compute resources for the wgpu backend.
#[derive(Debug)]
pub struct WgpuResources {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl WgpuResources {
    /// Initialize wgpu and create GPU resources.
    ///
    /// # Errors
    ///
    /// Returns an error if no compatible GPU adapter is found or if device
    /// creation fails.
    pub async fn new() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| GpuError::BackendUnavailable {
                backend: "wgpu",
            })?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("tpt-lexicon-gpu device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|_| GpuError::BackendUnavailable {
                backend: "wgpu",
            })?;

        Ok(Self { device, queue })
    }

    /// Return a reference to the device.
    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Return a reference to the queue.
    #[inline]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Tokenize `input` bytes by dispatching the BPE merge compute shader.
    ///
    /// Returns a vector of token IDs.
    ///
    /// # Panics
    ///
    /// Panics if the WGSL shader fails to compile or if a buffer mapping fails.
    pub fn tokenize(&self, input: &[u8]) -> Vec<u32> {
        // Currently a placeholder — actual compute shader dispatch not yet
        // implemented. Falls back to sequential CPU encoding.
        //
        // Future implementation will:
        // 1. Upload vocab merge rules to a GPU storage buffer
        // 2. Upload input bytes to a GPU storage buffer
        // 3. Dispatch a compute shader that performs parallel BPE merging
        // 4. Read back the resulting token IDs

        let _ = input;
        Vec::new()
    }
}
