#![allow(dead_code)]

use ash;
use std::ptr::null;

use crate::raw::device::Device;
use crate::raw::errors::SurfaceError;
use crate::raw::surface::Surface;

pub struct SwapChainConfig {
    pub min_image_count: u32,
    pub image_format: ash::vk::Format,
    pub image_extent: ash::vk::Extent2D,
    pub image_usage: ash::vk::ImageUsageFlags,
    pub present_mode: ash::vk::PresentModeKHR,
}

pub struct SwapChain {
    raw: ash::vk::SwapchainKHR,
}

impl SwapChain {
    pub fn extensions() -> Vec<&'static str> {
        // TODO: Clean this up
        vec![
            ash::extensions::khr::Swapchain::name().to_str().unwrap(),
            "VK_KHR_maintenance1",
            "VK_KHR_maintenance2",
            "VK_KHR_sampler_mirror_clamp_to_edge",
        ]
        //vec![ash::extensions::Swapchain::name().to_str().unwrap()]
    }

    /// Create new swap chain
    pub fn create(
        device: &Device,
        surface: ash::vk::SurfaceKHR,
        //surface: &Surface,
        old_swap_chain: Option<Self>,
        config: SwapChainConfig,
    ) -> Result<Self, SurfaceError> {
        let mut swap_chain = ash::vk::SwapchainKHR::null();
        let result = unsafe {
            device.swap_chain.as_ref().unwrap().create_swapchain_khr(
                device.raw.handle(),
                &ash::vk::SwapchainCreateInfoKHR {
                    s_type: ash::vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
                    p_next: null(),
                    flags: ash::vk::SwapchainCreateFlagsKHR::empty(),
                    surface,
                    //surface: surface.raw,
                    min_image_count: config.min_image_count,
                    image_format: config.image_format,
                    image_color_space: ash::vk::ColorSpaceKHR::SRGB_NONLINEAR,
                    image_extent: config.image_extent,
                    image_array_layers: 1,
                    image_usage: config.image_usage,
                    image_sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
                    queue_family_index_count: 0,
                    p_queue_family_indices: null(),
                    pre_transform: ash::vk::SurfaceTransformFlagsKHR::INHERIT,
                    composite_alpha: ash::vk::CompositeAlphaFlagsKHR::INHERIT,
                    present_mode: config.present_mode,
                    clipped: 1,
                    old_swapchain: old_swap_chain
                        .map_or(ash::vk::SwapchainKHR::null(), |swap_chain| swap_chain.raw),
                },
                null(),
                &mut swap_chain,
            )
        };

        match result {
            ash::vk::Result::SUCCESS => Ok(SwapChain { raw: swap_chain }),
            error => Err(SurfaceError::from_vk_result(error)),
        }
    }
}
