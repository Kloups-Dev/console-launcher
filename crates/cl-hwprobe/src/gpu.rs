use ash::vk;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuKind {
    Discrete,
    Integrated,
    Virtual,
    Cpu,
    Other,
}

impl GpuKind {
    /// Preference order when several GPUs are present.
    pub fn rank(self) -> u8 {
        match self {
            GpuKind::Discrete => 4,
            GpuKind::Integrated => 3,
            GpuKind::Virtual => 2,
            GpuKind::Other => 1,
            GpuKind::Cpu => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub kind: GpuKind,
    /// PCI vendor: 0x10DE NVIDIA, 0x1002 AMD, 0x8086 Intel.
    pub vendor_id: u32,
    pub vendor: String,
    /// Device-local memory. For integrated GPUs this is shared system memory.
    pub vram_mb: u64,
    /// Highest Vulkan version supported, e.g. "1.3.280".
    pub vulkan: String,
}

fn vendor_name(id: u32) -> &'static str {
    match id {
        0x10DE => "NVIDIA",
        0x1002 | 0x1022 => "AMD",
        0x8086 => "Intel",
        0x13B5 => "ARM",
        0x5143 => "Qualcomm",
        0x10005 => "Mesa (software)",
        _ => "unknown",
    }
}

pub fn detect_gpus() -> Result<Vec<GpuInfo>, String> {
    // SAFETY: loading the system Vulkan loader; failure is handled.
    let entry =
        unsafe { ash::Entry::load() }.map_err(|e| format!("Vulkan loader not found: {e}"))?;
    let app = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
    let info = vk::InstanceCreateInfo::default().application_info(&app);
    // SAFETY: valid create info, no layers or extensions.
    let instance = unsafe { entry.create_instance(&info, None) }
        .map_err(|e| format!("vkCreateInstance: {e}"))?;

    let result = (|| {
        // SAFETY: instance is valid for the duration of this closure.
        let devices =
            unsafe { instance.enumerate_physical_devices() }.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for dev in devices {
            // SAFETY: `dev` comes from the instance above.
            let props = unsafe { instance.get_physical_device_properties(dev) };
            let mem = unsafe { instance.get_physical_device_memory_properties(dev) };
            let name = props
                .device_name_as_c_str()
                .map(CStr::to_string_lossy)
                .map(|s| s.into_owned())
                .unwrap_or_default();
            let kind = match props.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => GpuKind::Discrete,
                vk::PhysicalDeviceType::INTEGRATED_GPU => GpuKind::Integrated,
                vk::PhysicalDeviceType::VIRTUAL_GPU => GpuKind::Virtual,
                vk::PhysicalDeviceType::CPU => GpuKind::Cpu,
                _ => GpuKind::Other,
            };
            let vram: u64 = mem.memory_heaps[..mem.memory_heap_count as usize]
                .iter()
                .filter(|h| h.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                .map(|h| h.size)
                .max()
                .unwrap_or(0);
            let v = props.api_version;
            out.push(GpuInfo {
                name,
                kind,
                vendor_id: props.vendor_id,
                vendor: vendor_name(props.vendor_id).to_owned(),
                vram_mb: vram / (1024 * 1024),
                vulkan: format!(
                    "{}.{}.{}",
                    vk::api_version_major(v),
                    vk::api_version_minor(v),
                    vk::api_version_patch(v)
                ),
            });
        }
        Ok(out)
    })();

    // SAFETY: no object created from the instance outlives it.
    unsafe { instance.destroy_instance(None) };
    result
}
