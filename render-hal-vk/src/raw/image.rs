#![allow(dead_code)]

use crate::raw::format::convert_format;
use ash;
use render_core::types::*;
use std::ptr;

pub type Type = ash::vk::ImageType;
pub type Extent3D = ash::vk::Extent3D;
pub type Layout = ash::vk::ImageLayout;
pub type Usage = ash::vk::ImageUsageFlags;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Kind {
    D1(u32),
    D2 { width: u32, height: u32 },
    D3 { width: u32, height: u32, depth: u32 },
}

impl Kind {
    /// Get type of the image.
    pub fn image_type(self) -> Type {
        match self {
            Kind::D1(_) => ash::vk::ImageType::TYPE_1D,
            Kind::D2 { .. } => ash::vk::ImageType::TYPE_2D,
            Kind::D3 { .. } => ash::vk::ImageType::TYPE_3D,
        }
    }

    /// Get extent of the image.
    pub fn extent(self) -> Extent3D {
        match self {
            Kind::D1(size) => Extent3D {
                width: size,
                height: 1,
                depth: 1,
            },
            Kind::D2 { width, height } => Extent3D {
                width,
                height,
                depth: 1,
            },
            Kind::D3 {
                width,
                height,
                depth,
            } => Extent3D {
                width,
                height,
                depth,
            },
        }
    }
}

pub struct Image;

pub fn convert_view_dimension_to_view_type(
    dimension: RenderViewDimension,
) -> ash::vk::ImageViewType {
    match dimension {
        RenderViewDimension::Buffer => unimplemented!(), // Buffer needs to be handled special
        RenderViewDimension::Tex1d => ash::vk::ImageViewType::TYPE_1D,
        RenderViewDimension::Tex1dArray => ash::vk::ImageViewType::TYPE_1D_ARRAY,
        RenderViewDimension::Tex2d => ash::vk::ImageViewType::TYPE_2D,
        RenderViewDimension::Tex2dArray => ash::vk::ImageViewType::TYPE_2D_ARRAY,
        RenderViewDimension::Tex3d => ash::vk::ImageViewType::TYPE_3D,
        RenderViewDimension::Cube => ash::vk::ImageViewType::CUBE,
        RenderViewDimension::CubeArray => ash::vk::ImageViewType::CUBE_ARRAY,
        _ => unimplemented!(),
    }
}

pub fn convert_texture_type_to_view_type(
    texture_type: RenderTextureType,
) -> ash::vk::ImageViewType {
    match texture_type {
        RenderTextureType::Tex1d => ash::vk::ImageViewType::TYPE_1D,
        RenderTextureType::Tex1dArray => ash::vk::ImageViewType::TYPE_1D_ARRAY,
        RenderTextureType::Tex2d => ash::vk::ImageViewType::TYPE_2D,
        RenderTextureType::Tex2dArray => ash::vk::ImageViewType::TYPE_2D_ARRAY,
        RenderTextureType::Tex3d => ash::vk::ImageViewType::TYPE_3D,
        RenderTextureType::Cube => ash::vk::ImageViewType::CUBE,
        RenderTextureType::CubeArray => ash::vk::ImageViewType::CUBE_ARRAY,
    }
}

pub fn get_image_create_info(
    desc: &RenderTextureDesc,
    initial_data: bool,
) -> ash::vk::ImageCreateInfo {
    let format = convert_format(desc.format, false /* typeless */);

    let (image_type, image_extent, image_layers) = match desc.texture_type {
        RenderTextureType::Tex1d => (
            ash::vk::ImageType::TYPE_1D,
            ash::vk::Extent3D {
                width: desc.width,
                height: 1,
                depth: 1,
            },
            1,
        ),
        RenderTextureType::Tex1dArray => (
            ash::vk::ImageType::TYPE_1D,
            ash::vk::Extent3D {
                width: desc.width,
                height: 1,
                depth: 1,
            },
            desc.elements,
        ),
        RenderTextureType::Tex2d => (
            ash::vk::ImageType::TYPE_2D,
            ash::vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: 1,
            },
            1,
        ),
        RenderTextureType::Tex2dArray => (
            ash::vk::ImageType::TYPE_2D,
            ash::vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: 1,
            },
            desc.elements,
        ),
        RenderTextureType::Tex3d => (
            ash::vk::ImageType::TYPE_3D,
            ash::vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: desc.depth as u32,
            },
            1,
        ),
        RenderTextureType::Cube => (
            ash::vk::ImageType::TYPE_2D,
            ash::vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: 1,
            },
            6,
        ),
        RenderTextureType::CubeArray => (
            ash::vk::ImageType::TYPE_2D,
            ash::vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: 1,
            },
            6 * desc.elements,
        ),
    };

    let mut image_usage = ash::vk::ImageUsageFlags::TRANSFER_DST; // Ideally make this 0 (need to PR ash)

    if initial_data {
        image_usage |= ash::vk::ImageUsageFlags::TRANSFER_DST;
    }

    if desc.bind_flags.contains(RenderBindFlags::SHADER_RESOURCE) {
        image_usage |= ash::vk::ImageUsageFlags::SAMPLED;
    }

    if desc.bind_flags.contains(RenderBindFlags::UNORDERED_ACCESS) {
        image_usage |= ash::vk::ImageUsageFlags::STORAGE;
    }

    if desc.bind_flags.contains(RenderBindFlags::RENDER_TARGET) {
        image_usage |= ash::vk::ImageUsageFlags::COLOR_ATTACHMENT;
        image_usage |= ash::vk::ImageUsageFlags::TRANSFER_DST; // Allow transfers to this surface
        image_usage |= ash::vk::ImageUsageFlags::TRANSFER_SRC; // Allow transfers from this surface
    }

    if desc.bind_flags.contains(RenderBindFlags::DEPTH_STENCIL) {
        image_usage |= ash::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        image_usage |= ash::vk::ImageUsageFlags::TRANSFER_DST; // Allow transfers to this surface
        image_usage |= ash::vk::ImageUsageFlags::TRANSFER_SRC; // Allow transfers from this surface
    }

    ash::vk::ImageCreateInfo {
        flags: match desc.texture_type {
            RenderTextureType::Cube => ash::vk::ImageCreateFlags::CUBE_COMPATIBLE,
            RenderTextureType::CubeArray => ash::vk::ImageCreateFlags::CUBE_COMPATIBLE,
            _ => ash::vk::ImageCreateFlags::empty(), // ImageCreateFlags::CREATE_MUTABLE_FORMAT
        },
        image_type,
        format,
        extent: image_extent,
        mip_levels: desc.levels as u32,
        array_layers: image_layers as u32,
        samples: ash::vk::SampleCountFlags::TYPE_1, // TODO: desc.sample_count
        tiling: match format {
            ash::vk::Format::R32G32B32_SFLOAT => ash::vk::ImageTiling::LINEAR,
            _ => ash::vk::ImageTiling::OPTIMAL,
        },
        usage: image_usage,
        sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
        initial_layout: match initial_data {
            true => ash::vk::ImageLayout::PREINITIALIZED,
            false => ash::vk::ImageLayout::UNDEFINED,
        },
        ..Default::default()
    }
}
