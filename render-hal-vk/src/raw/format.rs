use ash;
use render_core::format::{channel_format_has_depth, channel_format_has_stencil};
use render_core::state::{
    RenderBlendMode, RenderBlendOp, RenderCompareFunc, RenderCullMode, RenderFillMode,
    RenderStencilOp, RenderStencilSide,
};
use render_core::types::{
    RenderChannelFormat, RenderFormat, RenderFormatInfo, RenderPrimitiveType,
};
pub type Format = ash::vk::Format;

pub fn bool_to_vk(state: bool) -> ash::vk::Bool32 {
    match state {
        true => ash::vk::TRUE,
        false => ash::vk::FALSE,
    }
}

pub fn convert_format(format: RenderFormat, _typeless: bool) -> Format {
    match format {
        RenderFormat::Unknown => ash::vk::Format::UNDEFINED,

        RenderFormat::R4g4Unorm => ash::vk::Format::R4G4_UNORM_PACK8,
        RenderFormat::R4g4b4a4Unorm => ash::vk::Format::R4G4B4A4_UNORM_PACK16,
        RenderFormat::R5g6b5Unorm => ash::vk::Format::R5G6B5_UNORM_PACK16,
        RenderFormat::R5g5b5a1Unorm => ash::vk::Format::R5G5B5A1_UNORM_PACK16,

        RenderFormat::R8Unorm => ash::vk::Format::R8_UNORM,
        RenderFormat::R8Snorm => ash::vk::Format::R8_SNORM,
        RenderFormat::R8Srgb => ash::vk::Format::R8_SRGB,
        RenderFormat::R8Uint => ash::vk::Format::R8_UINT,
        RenderFormat::R8Sint => ash::vk::Format::R8_SINT,

        RenderFormat::R8g8Unorm => ash::vk::Format::R8G8_UNORM,
        RenderFormat::R8g8Snorm => ash::vk::Format::R8G8_SNORM,
        RenderFormat::R8g8Srgb => ash::vk::Format::R8G8_SRGB,
        RenderFormat::R8g8Uint => ash::vk::Format::R8G8_UINT,
        RenderFormat::R8g8Sint => ash::vk::Format::R8G8_SINT,

        RenderFormat::R8g8b8Unorm => ash::vk::Format::R8G8B8_UNORM,
        RenderFormat::R8g8b8Srgb => ash::vk::Format::R8G8B8_SRGB,

        RenderFormat::R8g8b8a8Unorm => ash::vk::Format::R8G8B8A8_UNORM,
        RenderFormat::R8g8b8a8Snorm => ash::vk::Format::R8G8B8A8_SNORM,
        RenderFormat::R8g8b8a8Srgb => ash::vk::Format::R8G8B8A8_SRGB,
        RenderFormat::R8g8b8a8Uint => ash::vk::Format::R8G8B8A8_UINT,
        RenderFormat::R8g8b8a8Sint => ash::vk::Format::R8G8B8A8_SINT,

        RenderFormat::B8g8r8a8Unorm => ash::vk::Format::B8G8R8A8_UNORM,
        RenderFormat::B8g8r8a8Srgb => ash::vk::Format::B8G8R8A8_SRGB,

        RenderFormat::R11g11b10Float => ash::vk::Format::B10G11R11_UFLOAT_PACK32,
        RenderFormat::R10g10b10a2Unorm => ash::vk::Format::A2R10G10B10_UNORM_PACK32,
        RenderFormat::R10g10b10a2Uint => ash::vk::Format::A2R10G10B10_UINT_PACK32,
        RenderFormat::R9g9b9e5Float => ash::vk::Format::E5B9G9R9_UFLOAT_PACK32,

        RenderFormat::R16Float => ash::vk::Format::R16_SFLOAT,
        RenderFormat::R16Unorm => ash::vk::Format::R16_UNORM,
        RenderFormat::R16Snorm => ash::vk::Format::R16_SNORM,
        RenderFormat::R16Uint => ash::vk::Format::R16_UINT,
        RenderFormat::R16Sint => ash::vk::Format::R16_SINT,

        RenderFormat::R16g16Float => ash::vk::Format::R16G16_SFLOAT,
        RenderFormat::R16g16Unorm => ash::vk::Format::R16G16_UNORM,
        RenderFormat::R16g16Snorm => ash::vk::Format::R16G16_SNORM,
        RenderFormat::R16g16Uint => ash::vk::Format::R16G16_UINT,
        RenderFormat::R16g16Sint => ash::vk::Format::R16G16_SINT,

        RenderFormat::R16g16b16a16Float => ash::vk::Format::R16G16B16A16_SFLOAT,
        RenderFormat::R16g16b16a16Unorm => ash::vk::Format::R16G16B16A16_UNORM,
        RenderFormat::R16g16b16a16Snorm => ash::vk::Format::R16G16B16A16_SNORM,
        RenderFormat::R16g16b16a16Uint => ash::vk::Format::R16G16B16A16_UINT,
        RenderFormat::R16g16b16a16Sint => ash::vk::Format::R16G16B16A16_SINT,

        RenderFormat::R32Float => ash::vk::Format::R32_SFLOAT,
        RenderFormat::R32Uint => ash::vk::Format::R32_UINT,
        RenderFormat::R32Sint => ash::vk::Format::R32_SINT,

        RenderFormat::R32g32Float => ash::vk::Format::R32G32_SFLOAT,
        RenderFormat::R32g32Uint => ash::vk::Format::R32G32_UINT,
        RenderFormat::R32g32Sint => ash::vk::Format::R32G32_SINT,

        RenderFormat::R32g32b32Float => ash::vk::Format::R32G32B32_SFLOAT,
        RenderFormat::R32g32b32Uint => ash::vk::Format::R32G32B32_UINT,
        RenderFormat::R32g32b32Sint => ash::vk::Format::R32G32B32_SINT,

        RenderFormat::R32g32b32a32Float => ash::vk::Format::R32G32B32A32_SFLOAT,
        RenderFormat::R32g32b32a32Uint => ash::vk::Format::R32G32B32A32_UINT,
        RenderFormat::R32g32b32a32Sint => ash::vk::Format::R32G32B32A32_SINT,

        RenderFormat::Bc1Unorm => ash::vk::Format::BC1_RGB_UNORM_BLOCK,
        RenderFormat::Bc1Srgb => ash::vk::Format::BC1_RGB_SRGB_BLOCK,
        RenderFormat::Bc1aUnorm => ash::vk::Format::BC1_RGBA_UNORM_BLOCK,
        RenderFormat::Bc1aSrgb => ash::vk::Format::BC1_RGBA_SRGB_BLOCK,
        RenderFormat::Bc2Unorm => ash::vk::Format::BC2_UNORM_BLOCK,
        RenderFormat::Bc2Srgb => ash::vk::Format::BC2_SRGB_BLOCK,
        RenderFormat::Bc3Unorm => ash::vk::Format::BC3_UNORM_BLOCK,
        RenderFormat::Bc3Srgb => ash::vk::Format::BC3_SRGB_BLOCK,
        RenderFormat::Bc4Unorm => ash::vk::Format::BC4_UNORM_BLOCK,
        RenderFormat::Bc4Snorm => ash::vk::Format::BC4_SNORM_BLOCK,
        RenderFormat::Bc5Unorm => ash::vk::Format::BC5_UNORM_BLOCK,
        RenderFormat::Bc5Snorm => ash::vk::Format::BC5_SNORM_BLOCK,
        RenderFormat::Bc6uFloat => ash::vk::Format::BC6H_UFLOAT_BLOCK,
        RenderFormat::Bc6sFloat => ash::vk::Format::BC6H_SFLOAT_BLOCK,
        RenderFormat::Bc7Unorm => ash::vk::Format::BC7_UNORM_BLOCK,
        RenderFormat::Bc7Srgb => ash::vk::Format::BC7_SRGB_BLOCK,

        RenderFormat::D24UnormS8Uint => ash::vk::Format::D24_UNORM_S8_UINT,
        RenderFormat::D32FloatS8Uint => ash::vk::Format::D32_SFLOAT_S8_UINT,
        RenderFormat::D16Unorm => ash::vk::Format::D16_UNORM,
        RenderFormat::D32Float => ash::vk::Format::D32_SFLOAT,

        RenderFormat::Astc4x4Unorm => ash::vk::Format::ASTC_4X4_UNORM_BLOCK,
        RenderFormat::Astc4x4Srgb => ash::vk::Format::ASTC_4X4_SRGB_BLOCK,
        RenderFormat::Astc5x4Unorm => ash::vk::Format::ASTC_5X4_UNORM_BLOCK,
        RenderFormat::Astc5x4Srgb => ash::vk::Format::ASTC_5X4_SRGB_BLOCK,
        RenderFormat::Astc5x5Unorm => ash::vk::Format::ASTC_5X5_UNORM_BLOCK,
        RenderFormat::Astc5x5Srgb => ash::vk::Format::ASTC_5X5_SRGB_BLOCK,
        RenderFormat::Astc6x5Unorm => ash::vk::Format::ASTC_6X5_UNORM_BLOCK,
        RenderFormat::Astc6x5Srgb => ash::vk::Format::ASTC_6X5_SRGB_BLOCK,
        RenderFormat::Astc6x6Unorm => ash::vk::Format::ASTC_6X6_UNORM_BLOCK,
        RenderFormat::Astc6x6Srgb => ash::vk::Format::ASTC_6X6_SRGB_BLOCK,
        RenderFormat::Astc8x5Unorm => ash::vk::Format::ASTC_8X5_UNORM_BLOCK,
        RenderFormat::Astc8x5Srgb => ash::vk::Format::ASTC_8X5_SRGB_BLOCK,
        RenderFormat::Astc8x6Unorm => ash::vk::Format::ASTC_8X6_UNORM_BLOCK,
        RenderFormat::Astc8x6Srgb => ash::vk::Format::ASTC_8X6_SRGB_BLOCK,
        RenderFormat::Astc8x8Unorm => ash::vk::Format::ASTC_8X8_UNORM_BLOCK,
        RenderFormat::Astc8x8Srgb => ash::vk::Format::ASTC_8X8_SRGB_BLOCK,
        RenderFormat::Astc10x5Unorm => ash::vk::Format::ASTC_10X5_UNORM_BLOCK,
        RenderFormat::Astc10x5Srgb => ash::vk::Format::ASTC_10X5_SRGB_BLOCK,
        RenderFormat::Astc10x6Unorm => ash::vk::Format::ASTC_10X6_UNORM_BLOCK,
        RenderFormat::Astc10x6Srgb => ash::vk::Format::ASTC_10X6_SRGB_BLOCK,
        RenderFormat::Astc10x8Unorm => ash::vk::Format::ASTC_10X8_UNORM_BLOCK,
        RenderFormat::Astc10x8Srgb => ash::vk::Format::ASTC_10X8_SRGB_BLOCK,
        RenderFormat::Astc10x10Unorm => ash::vk::Format::ASTC_10X10_UNORM_BLOCK,
        RenderFormat::Astc10x10Srgb => ash::vk::Format::ASTC_10X10_SRGB_BLOCK,
        RenderFormat::Astc12x10Unorm => ash::vk::Format::ASTC_12X10_UNORM_BLOCK,
        RenderFormat::Astc12x10Srgb => ash::vk::Format::ASTC_12X10_SRGB_BLOCK,
        RenderFormat::Astc12x12Unorm => ash::vk::Format::ASTC_12X12_UNORM_BLOCK,
        RenderFormat::Astc12x12Srgb => ash::vk::Format::ASTC_12X12_SRGB_BLOCK,
    }
}

pub(crate) fn get_image_aspect_flags(
    format: RenderFormat,
    ignore_stencil: bool,
) -> ash::vk::ImageAspectFlags {
    let channel_format: RenderChannelFormat = format.into();
    let mut flags: ash::vk::ImageAspectFlags = ash::vk::ImageAspectFlags::empty();

    if channel_format_has_depth(channel_format) {
        flags |= ash::vk::ImageAspectFlags::DEPTH;
    }

    if channel_format_has_stencil(channel_format) && !ignore_stencil {
        flags |= ash::vk::ImageAspectFlags::STENCIL;
    }

    if flags == ash::vk::ImageAspectFlags::empty() {
        flags |= ash::vk::ImageAspectFlags::COLOR;
    }

    flags
}

pub fn get_primitive_topology(topology: RenderPrimitiveType) -> ash::vk::PrimitiveTopology {
    match topology {
        RenderPrimitiveType::PointList => ash::vk::PrimitiveTopology::POINT_LIST,
        RenderPrimitiveType::LineList => ash::vk::PrimitiveTopology::LINE_LIST,
        RenderPrimitiveType::LineStrip => ash::vk::PrimitiveTopology::LINE_STRIP,
        RenderPrimitiveType::TriangleList => ash::vk::PrimitiveTopology::TRIANGLE_LIST,
        RenderPrimitiveType::TriangleStrip => ash::vk::PrimitiveTopology::TRIANGLE_STRIP,
        RenderPrimitiveType::TrianglePatch => ash::vk::PrimitiveTopology::PATCH_LIST,
        RenderPrimitiveType::RectList => unimplemented!(),
        RenderPrimitiveType::QuadList => unimplemented!(),
    }
}

pub fn get_polygon_mode(fill_mode: RenderFillMode) -> ash::vk::PolygonMode {
    match fill_mode {
        RenderFillMode::Solid => ash::vk::PolygonMode::FILL,
        RenderFillMode::WireFrame => ash::vk::PolygonMode::LINE,
    }
}

pub fn get_cull_mode(cull_mode: RenderCullMode) -> ash::vk::CullModeFlags {
    match cull_mode {
        RenderCullMode::None => ash::vk::CullModeFlags::NONE,
        RenderCullMode::Front => ash::vk::CullModeFlags::FRONT,
        RenderCullMode::Back => ash::vk::CullModeFlags::BACK,
    }
}

pub fn get_blend_factor(mode: RenderBlendMode) -> ash::vk::BlendFactor {
    match mode {
        RenderBlendMode::Zero => ash::vk::BlendFactor::ZERO,
        RenderBlendMode::One => ash::vk::BlendFactor::ONE,
        RenderBlendMode::SourceColor => ash::vk::BlendFactor::SRC_COLOR,
        RenderBlendMode::InvSourceColor => ash::vk::BlendFactor::ONE_MINUS_SRC_COLOR,
        RenderBlendMode::SourceAlpha => ash::vk::BlendFactor::SRC_ALPHA,
        RenderBlendMode::InvSourceAlpha => ash::vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        RenderBlendMode::DestColor => ash::vk::BlendFactor::DST_COLOR,
        RenderBlendMode::InvDestColor => ash::vk::BlendFactor::ONE_MINUS_DST_COLOR,
        RenderBlendMode::DestAlpha => ash::vk::BlendFactor::DST_ALPHA,
        RenderBlendMode::InvDestAlpha => ash::vk::BlendFactor::ONE_MINUS_DST_ALPHA,
        RenderBlendMode::SourceAlphaSaturate => ash::vk::BlendFactor::SRC_ALPHA_SATURATE,
        RenderBlendMode::Constant => ash::vk::BlendFactor::CONSTANT_COLOR,
        RenderBlendMode::InvConstant => ash::vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
        RenderBlendMode::Source1Color => ash::vk::BlendFactor::SRC1_COLOR,
        RenderBlendMode::InvSource1Color => ash::vk::BlendFactor::ONE_MINUS_SRC1_COLOR,
        RenderBlendMode::Source1Alpha => ash::vk::BlendFactor::SRC1_ALPHA,
        RenderBlendMode::InvSource1Alpha => ash::vk::BlendFactor::ONE_MINUS_SRC1_ALPHA,
    }
}

pub fn get_blend_op(op: RenderBlendOp) -> ash::vk::BlendOp {
    match op {
        RenderBlendOp::Add => ash::vk::BlendOp::ADD,
        RenderBlendOp::Subtract => ash::vk::BlendOp::SUBTRACT,
        RenderBlendOp::RevSubtract => ash::vk::BlendOp::REVERSE_SUBTRACT,
        RenderBlendOp::Min => ash::vk::BlendOp::MIN,
        RenderBlendOp::Max => ash::vk::BlendOp::MAX,
    }
}

pub fn get_stencil_op(op: RenderStencilOp) -> ash::vk::StencilOp {
    match op {
        RenderStencilOp::Keep => ash::vk::StencilOp::KEEP,
        RenderStencilOp::Zero => ash::vk::StencilOp::ZERO,
        RenderStencilOp::Replace => ash::vk::StencilOp::REPLACE,
        RenderStencilOp::IncrementSaturate => ash::vk::StencilOp::INCREMENT_AND_CLAMP,
        RenderStencilOp::IncrementWrap => ash::vk::StencilOp::INCREMENT_AND_WRAP,
        RenderStencilOp::DecrementSaturate => ash::vk::StencilOp::DECREMENT_AND_CLAMP,
        RenderStencilOp::DecrementWrap => ash::vk::StencilOp::DECREMENT_AND_WRAP,
        RenderStencilOp::Invert => ash::vk::StencilOp::INVERT,
    }
}

pub fn get_compare_op(func: RenderCompareFunc) -> ash::vk::CompareOp {
    match func {
        RenderCompareFunc::Never => ash::vk::CompareOp::NEVER,
        RenderCompareFunc::Less => ash::vk::CompareOp::LESS,
        RenderCompareFunc::Equal => ash::vk::CompareOp::EQUAL,
        RenderCompareFunc::LessEqual => ash::vk::CompareOp::LESS_OR_EQUAL,
        RenderCompareFunc::Greater => ash::vk::CompareOp::GREATER,
        RenderCompareFunc::NotEqual => ash::vk::CompareOp::NOT_EQUAL,
        RenderCompareFunc::GreaterEqual => ash::vk::CompareOp::GREATER_OR_EQUAL,
        RenderCompareFunc::Always => ash::vk::CompareOp::ALWAYS,
    }
}

pub fn get_stencil_op_state(
    desc: &RenderStencilSide,
    read_mask: u8,
    write_mask: u8,
) -> ash::vk::StencilOpState {
    ash::vk::StencilOpState::builder()
        .fail_op(get_stencil_op(desc.fail_op))
        .pass_op(get_stencil_op(desc.pass_op))
        .depth_fail_op(get_stencil_op(desc.depth_fail_op))
        .compare_op(get_compare_op(desc.func))
        .compare_mask(read_mask as u32)
        .write_mask(write_mask as u32)
        .reference(0) // set as dynamic state
        .build()
}
