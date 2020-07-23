use crate::types::{
    RenderBindFlags, RenderChannelFormat, RenderFormat, RenderFormatInfo, RenderNumericFormat,
    RenderPrimitiveType, RenderResourceStates, RenderTextureLayoutInfo, RenderTextureType,
    RenderViewDimension, RENDERCHANNELFORMAT_COUNT, RENDERFORMAT_COUNT, RENDERNUMERICFORMAT_COUNT,
};
use crate::utilities::align_forward;
use enum_primitive::FromPrimitive;
use strum::EnumCount;

impl Into<RenderChannelFormat> for RenderFormat {
    fn into(self) -> RenderChannelFormat {
        match self {
            RenderFormat::Unknown => RenderChannelFormat::Unknown,

            RenderFormat::R4g4Unorm => RenderChannelFormat::R4G4,
            RenderFormat::R4g4b4a4Unorm => RenderChannelFormat::R4G4B4A4,
            RenderFormat::R5g6b5Unorm => RenderChannelFormat::R5G6B5,
            RenderFormat::R5g5b5a1Unorm => RenderChannelFormat::R5G5B5A1,

            RenderFormat::R8Unorm => RenderChannelFormat::R8,
            RenderFormat::R8Snorm => RenderChannelFormat::R8,
            RenderFormat::R8Srgb => RenderChannelFormat::R8,
            RenderFormat::R8Uint => RenderChannelFormat::R8,
            RenderFormat::R8Sint => RenderChannelFormat::R8,

            RenderFormat::R8g8Unorm => RenderChannelFormat::R8G8,
            RenderFormat::R8g8Snorm => RenderChannelFormat::R8G8,
            RenderFormat::R8g8Srgb => RenderChannelFormat::R8G8,
            RenderFormat::R8g8Uint => RenderChannelFormat::R8G8,
            RenderFormat::R8g8Sint => RenderChannelFormat::R8G8,

            RenderFormat::R8g8b8Unorm => RenderChannelFormat::R8G8B8,
            RenderFormat::R8g8b8Srgb => RenderChannelFormat::R8G8B8,

            RenderFormat::R8g8b8a8Unorm => RenderChannelFormat::R8G8B8A8,
            RenderFormat::R8g8b8a8Snorm => RenderChannelFormat::R8G8B8A8,
            RenderFormat::R8g8b8a8Srgb => RenderChannelFormat::R8G8B8A8,
            RenderFormat::R8g8b8a8Uint => RenderChannelFormat::R8G8B8A8,
            RenderFormat::R8g8b8a8Sint => RenderChannelFormat::R8G8B8A8,

            RenderFormat::B8g8r8a8Unorm => RenderChannelFormat::B8G8R8A8,
            RenderFormat::B8g8r8a8Srgb => RenderChannelFormat::B8G8R8A8,

            RenderFormat::R11g11b10Float => RenderChannelFormat::R11G11B10,
            RenderFormat::R10g10b10a2Unorm => RenderChannelFormat::R10G10B10A2,
            RenderFormat::R10g10b10a2Uint => RenderChannelFormat::R10G10B10A2,
            RenderFormat::R9g9b9e5Float => RenderChannelFormat::R9G9B9E5,

            RenderFormat::R16Float => RenderChannelFormat::R16,
            RenderFormat::R16Unorm => RenderChannelFormat::R16,
            RenderFormat::R16Snorm => RenderChannelFormat::R16,
            RenderFormat::R16Uint => RenderChannelFormat::R16,
            RenderFormat::R16Sint => RenderChannelFormat::R16,

            RenderFormat::R16g16Float => RenderChannelFormat::R16G16,
            RenderFormat::R16g16Unorm => RenderChannelFormat::R16G16,
            RenderFormat::R16g16Snorm => RenderChannelFormat::R16G16,
            RenderFormat::R16g16Uint => RenderChannelFormat::R16G16,
            RenderFormat::R16g16Sint => RenderChannelFormat::R16G16,

            RenderFormat::R16g16b16a16Float => RenderChannelFormat::R16G16B16A16,
            RenderFormat::R16g16b16a16Unorm => RenderChannelFormat::R16G16B16A16,
            RenderFormat::R16g16b16a16Snorm => RenderChannelFormat::R16G16B16A16,
            RenderFormat::R16g16b16a16Uint => RenderChannelFormat::R16G16B16A16,
            RenderFormat::R16g16b16a16Sint => RenderChannelFormat::R16G16B16A16,

            RenderFormat::R32Float => RenderChannelFormat::R32,
            RenderFormat::R32Uint => RenderChannelFormat::R32,
            RenderFormat::R32Sint => RenderChannelFormat::R32,

            RenderFormat::R32g32Float => RenderChannelFormat::R32G32,
            RenderFormat::R32g32Uint => RenderChannelFormat::R32G32,
            RenderFormat::R32g32Sint => RenderChannelFormat::R32G32,

            RenderFormat::R32g32b32Float => RenderChannelFormat::R32G32B32,
            RenderFormat::R32g32b32Uint => RenderChannelFormat::R32G32B32,
            RenderFormat::R32g32b32Sint => RenderChannelFormat::R32G32B32,

            RenderFormat::R32g32b32a32Float => RenderChannelFormat::R32G32B32A32,
            RenderFormat::R32g32b32a32Uint => RenderChannelFormat::R32G32B32A32,
            RenderFormat::R32g32b32a32Sint => RenderChannelFormat::R32G32B32A32,

            RenderFormat::Bc1Unorm => RenderChannelFormat::BC1,
            RenderFormat::Bc1Srgb => RenderChannelFormat::BC1,
            RenderFormat::Bc1aUnorm => RenderChannelFormat::BC1A,
            RenderFormat::Bc1aSrgb => RenderChannelFormat::BC1A,
            RenderFormat::Bc2Unorm => RenderChannelFormat::BC2,
            RenderFormat::Bc2Srgb => RenderChannelFormat::BC2,
            RenderFormat::Bc3Unorm => RenderChannelFormat::BC3,
            RenderFormat::Bc3Srgb => RenderChannelFormat::BC3,
            RenderFormat::Bc4Unorm => RenderChannelFormat::BC4,
            RenderFormat::Bc4Snorm => RenderChannelFormat::BC4,
            RenderFormat::Bc5Unorm => RenderChannelFormat::BC5,
            RenderFormat::Bc5Snorm => RenderChannelFormat::BC5,
            RenderFormat::Bc6uFloat => RenderChannelFormat::BC6U,
            RenderFormat::Bc6sFloat => RenderChannelFormat::BC6S,
            RenderFormat::Bc7Unorm => RenderChannelFormat::BC7,
            RenderFormat::Bc7Srgb => RenderChannelFormat::BC7,

            RenderFormat::Astc4x4Unorm => RenderChannelFormat::Astc4x4,
            RenderFormat::Astc4x4Srgb => RenderChannelFormat::Astc4x4,
            RenderFormat::Astc5x4Unorm => RenderChannelFormat::Astc5x4,
            RenderFormat::Astc5x4Srgb => RenderChannelFormat::Astc5x4,
            RenderFormat::Astc5x5Unorm => RenderChannelFormat::Astc5x5,
            RenderFormat::Astc5x5Srgb => RenderChannelFormat::Astc5x5,
            RenderFormat::Astc6x5Unorm => RenderChannelFormat::Astc6x5,
            RenderFormat::Astc6x5Srgb => RenderChannelFormat::Astc6x5,
            RenderFormat::Astc6x6Unorm => RenderChannelFormat::Astc6x6,
            RenderFormat::Astc6x6Srgb => RenderChannelFormat::Astc6x6,
            RenderFormat::Astc8x5Unorm => RenderChannelFormat::Astc8x5,
            RenderFormat::Astc8x5Srgb => RenderChannelFormat::Astc8x5,
            RenderFormat::Astc8x6Unorm => RenderChannelFormat::Astc8x6,
            RenderFormat::Astc8x6Srgb => RenderChannelFormat::Astc8x6,
            RenderFormat::Astc8x8Unorm => RenderChannelFormat::Astc8x8,
            RenderFormat::Astc8x8Srgb => RenderChannelFormat::Astc8x8,
            RenderFormat::Astc10x5Unorm => RenderChannelFormat::Astc10x5,
            RenderFormat::Astc10x5Srgb => RenderChannelFormat::Astc10x5,
            RenderFormat::Astc10x6Unorm => RenderChannelFormat::Astc10x6,
            RenderFormat::Astc10x6Srgb => RenderChannelFormat::Astc10x6,
            RenderFormat::Astc10x8Unorm => RenderChannelFormat::Astc10x8,
            RenderFormat::Astc10x8Srgb => RenderChannelFormat::Astc10x8,
            RenderFormat::Astc10x10Unorm => RenderChannelFormat::Astc10x10,
            RenderFormat::Astc10x10Srgb => RenderChannelFormat::Astc10x10,
            RenderFormat::Astc12x10Unorm => RenderChannelFormat::Astc12x10,
            RenderFormat::Astc12x10Srgb => RenderChannelFormat::Astc12x10,
            RenderFormat::Astc12x12Unorm => RenderChannelFormat::Astc12x12,
            RenderFormat::Astc12x12Srgb => RenderChannelFormat::Astc12x12,

            RenderFormat::D24UnormS8Uint => RenderChannelFormat::D24S8,
            RenderFormat::D32FloatS8Uint => RenderChannelFormat::D32S8,
            RenderFormat::D16Unorm => RenderChannelFormat::D16,
            RenderFormat::D32Float => RenderChannelFormat::D32,
        }
    }
}
impl Into<RenderNumericFormat> for RenderFormat {
    fn into(self) -> RenderNumericFormat {
        match self {
            RenderFormat::Unknown => RenderNumericFormat::Unknown,

            RenderFormat::R4g4Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R4g4b4a4Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R5g6b5Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R5g5b5a1Unorm => RenderNumericFormat::Unorm,

            RenderFormat::R8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R8Snorm => RenderNumericFormat::Snorm,
            RenderFormat::R8Srgb => RenderNumericFormat::Srgb,
            RenderFormat::R8Uint => RenderNumericFormat::Uint,
            RenderFormat::R8Sint => RenderNumericFormat::Sint,

            RenderFormat::R8g8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R8g8Snorm => RenderNumericFormat::Snorm,
            RenderFormat::R8g8Srgb => RenderNumericFormat::Srgb,
            RenderFormat::R8g8Uint => RenderNumericFormat::Uint,
            RenderFormat::R8g8Sint => RenderNumericFormat::Sint,

            RenderFormat::R8g8b8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R8g8b8Srgb => RenderNumericFormat::Srgb,

            RenderFormat::R8g8b8a8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R8g8b8a8Snorm => RenderNumericFormat::Snorm,
            RenderFormat::R8g8b8a8Srgb => RenderNumericFormat::Srgb,
            RenderFormat::R8g8b8a8Uint => RenderNumericFormat::Uint,
            RenderFormat::R8g8b8a8Sint => RenderNumericFormat::Sint,

            RenderFormat::B8g8r8a8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::B8g8r8a8Srgb => RenderNumericFormat::Srgb,

            RenderFormat::R11g11b10Float => RenderNumericFormat::Float,
            RenderFormat::R10g10b10a2Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R10g10b10a2Uint => RenderNumericFormat::Uint,
            RenderFormat::R9g9b9e5Float => RenderNumericFormat::Float,

            RenderFormat::R16Float => RenderNumericFormat::Float,
            RenderFormat::R16Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R16Snorm => RenderNumericFormat::Snorm,
            RenderFormat::R16Uint => RenderNumericFormat::Uint,
            RenderFormat::R16Sint => RenderNumericFormat::Sint,

            RenderFormat::R16g16Float => RenderNumericFormat::Float,
            RenderFormat::R16g16Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R16g16Snorm => RenderNumericFormat::Snorm,
            RenderFormat::R16g16Uint => RenderNumericFormat::Uint,
            RenderFormat::R16g16Sint => RenderNumericFormat::Sint,

            RenderFormat::R16g16b16a16Float => RenderNumericFormat::Float,
            RenderFormat::R16g16b16a16Unorm => RenderNumericFormat::Unorm,
            RenderFormat::R16g16b16a16Snorm => RenderNumericFormat::Snorm,
            RenderFormat::R16g16b16a16Uint => RenderNumericFormat::Uint,
            RenderFormat::R16g16b16a16Sint => RenderNumericFormat::Sint,

            RenderFormat::R32Float => RenderNumericFormat::Float,
            RenderFormat::R32Uint => RenderNumericFormat::Uint,
            RenderFormat::R32Sint => RenderNumericFormat::Sint,

            RenderFormat::R32g32Float => RenderNumericFormat::Float,
            RenderFormat::R32g32Uint => RenderNumericFormat::Uint,
            RenderFormat::R32g32Sint => RenderNumericFormat::Sint,

            RenderFormat::R32g32b32Float => RenderNumericFormat::Float,
            RenderFormat::R32g32b32Uint => RenderNumericFormat::Uint,
            RenderFormat::R32g32b32Sint => RenderNumericFormat::Sint,

            RenderFormat::R32g32b32a32Float => RenderNumericFormat::Float,
            RenderFormat::R32g32b32a32Uint => RenderNumericFormat::Uint,
            RenderFormat::R32g32b32a32Sint => RenderNumericFormat::Sint,

            RenderFormat::Bc1Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc1Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Bc1aUnorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc1aSrgb => RenderNumericFormat::Srgb,
            RenderFormat::Bc2Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc2Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Bc3Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc3Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Bc4Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc4Snorm => RenderNumericFormat::Snorm,
            RenderFormat::Bc5Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc5Snorm => RenderNumericFormat::Snorm,
            RenderFormat::Bc6uFloat => RenderNumericFormat::Float,
            RenderFormat::Bc6sFloat => RenderNumericFormat::Float,
            RenderFormat::Bc7Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Bc7Srgb => RenderNumericFormat::Srgb,

            RenderFormat::Astc4x4Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc4x4Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc5x4Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc5x4Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc5x5Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc5x5Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc6x5Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc6x5Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc6x6Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc6x6Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc8x5Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc8x5Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc8x6Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc8x6Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc8x8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc8x8Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc10x5Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc10x5Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc10x6Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc10x6Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc10x8Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc10x8Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc10x10Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc10x10Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc12x10Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc12x10Srgb => RenderNumericFormat::Srgb,
            RenderFormat::Astc12x12Unorm => RenderNumericFormat::Unorm,
            RenderFormat::Astc12x12Srgb => RenderNumericFormat::Srgb,

            RenderFormat::D24UnormS8Uint => RenderNumericFormat::Uint,
            RenderFormat::D32FloatS8Uint => RenderNumericFormat::Float,
            RenderFormat::D16Unorm => RenderNumericFormat::Unorm,
            RenderFormat::D32Float => RenderNumericFormat::Float,
        }
    }
}

bitflags! {
    pub struct RenderFormatCapability: u32 {
        /// The format is unsupported
        const UNSUPPORTED = 0;

        /// The format can be read in shaders
        const SHADER_READ = 1<<0;

        /// The format can be read in shaders with filtering (bilinear etc)
        const SHADER_READ_FILTERABLE = 1<<1;

        /// The format can be written to from a shader
        const SHADER_WRITE = 1<<2;

        /// The format can be used as a render target for writing
        const RENDER_TARGET_WRITE = 1<<3;

        /// The format can be used as a render target for writing & blending
        const RENDER_TARGET_BLEND = 1<<4;

        /// The format can be used as a depth target
        const DEPTH_TARGET = 1<<5;

        /// The format can be used as a stencil target
        const STENCIL_TARGET = 1<<6;

        /// The format can be used with multi-sample anti-aliasing
        const MSAA = 1<<7;

        /// The format can be copied
        const COPY = 1<<8;

        /// The format can have mip maps generated by our HAL
        const GENERATE_MIPS = 1<<9;

        /// The format can be used for textures of type `RenderTextureType::Tex1d`
        const TEXTURE_1D = 1<<11;

        /// The format can be used for textures of type `RenderTextureType::Tex1dArray`
        const TEXTURE_1D_ARRAY = 1<<12;

        /// The format can be used for textures of type `RenderTextureType::Tex2d`
        const TEXTURE_2D = 1<<13;

        /// The format can be used for textures of type `RenderTextureType::Tex2dArray`
        const TEXTURE_2D_ARRAY = 1<<14;

        /// The format can be used for textures of type `RenderTextureType::Cube`
        const TEXTURE_CUBE = 1<<15;

        /// The format can be used for textures of type `RenderTextureType::CubeArray`
        const TEXTURE_CUBE_ARRAY = 1<<16;

        /// The format can be used for textures of type `RenderTextureType::Tex3d`
        const TEXTURE_3D = 1<<17;
    }
}

impl Default for RenderFormatCapability {
    fn default() -> Self {
        RenderFormatCapability::UNSUPPORTED
    }
}

pub fn build_format_capability_text(format: RenderFormat, caps: RenderFormatCapability) -> String {
    let mut result = String::with_capacity(1046);

    result.push_str(&format!("{:?}", format));
    result.push_str(": ");

    for _ in 0..40 {
        result.push(' ');
    }

    if caps == RenderFormatCapability::UNSUPPORTED {
        result.push_str("Unsupported");
    } else {
        if caps.contains(RenderFormatCapability::SHADER_READ) {
            result.push_str("ShaderRead ");
        } else {
            result.push_str("           ");
        }

        if caps.contains(RenderFormatCapability::SHADER_READ_FILTERABLE) {
            result.push_str("ShaderReadFilterable ");
        } else {
            result.push_str("                     ");
        }

        if caps.contains(RenderFormatCapability::SHADER_WRITE) {
            result.push_str("ShaderWrite ");
        } else {
            result.push_str("            ");
        }

        if caps.contains(RenderFormatCapability::RENDER_TARGET_WRITE) {
            result.push_str("RenderTargetWrite ");
        } else {
            result.push_str("                  ");
        }

        if caps.contains(RenderFormatCapability::RENDER_TARGET_BLEND) {
            result.push_str("RenderTargetBlend ");
        } else {
            result.push_str("                  ");
        }

        if caps.contains(RenderFormatCapability::DEPTH_TARGET) {
            result.push_str("DepthTarget ");
        } else {
            result.push_str("            ");
        }

        if caps.contains(RenderFormatCapability::STENCIL_TARGET) {
            result.push_str("StencilTarget ");
        } else {
            result.push_str("              ");
        }

        if caps.contains(RenderFormatCapability::MSAA) {
            result.push_str("MSAA ");
        } else {
            result.push_str("     ");
        }

        if caps.contains(RenderFormatCapability::COPY) {
            result.push_str("Copy ");
        } else {
            result.push_str("     ");
        }

        if caps.contains(RenderFormatCapability::GENERATE_MIPS) {
            result.push_str("GenerateMipmaps ");
        } else {
            result.push_str("                ");
        }

        /*if caps.contains(RenderFormatCapability::PRESENT) {
            result.push_str("Present ");
        } else {
            result.push_str("        ");
        }*/
    }

    result
}

pub fn build_format(
    channel_format: RenderChannelFormat,
    numeric_format: RenderNumericFormat,
    validate: bool,
) -> RenderFormat {
    for i in 0..RENDERFORMAT_COUNT {
        let format = RenderFormat::from_u32(i as u32).unwrap();
        let channel_check: RenderChannelFormat = format.into();
        let numeric_check: RenderNumericFormat = format.into();
        if channel_check == channel_format && numeric_check == numeric_format {
            return format;
        }
    }

    if validate {
        panic!("Format is invalid!");
    } else {
        RenderFormat::Unknown
    }
}

pub fn get_render_format_info(format: RenderFormat) -> RenderFormatInfo {
    let mut info: RenderFormatInfo = Default::default();

    match format {
        RenderFormat::R32g32b32a32Float
        | RenderFormat::R32g32b32a32Uint
        | RenderFormat::R32g32b32a32Sint => {
            info.red_bits = 32;
            info.green_bits = 32;
            info.blue_bits = 32;
            info.alpha_bits = 32;
        }
        RenderFormat::R32g32b32Float
        | RenderFormat::R32g32b32Uint
        | RenderFormat::R32g32b32Sint => {
            info.red_bits = 32;
            info.green_bits = 32;
            info.blue_bits = 32;
        }
        RenderFormat::R16g16b16a16Float
        | RenderFormat::R16g16b16a16Unorm
        | RenderFormat::R16g16b16a16Uint
        | RenderFormat::R16g16b16a16Snorm
        | RenderFormat::R16g16b16a16Sint => {
            info.red_bits = 16;
            info.green_bits = 16;
            info.blue_bits = 16;
            info.alpha_bits = 16;
        }
        RenderFormat::R32g32Float | RenderFormat::R32g32Uint | RenderFormat::R32g32Sint => {
            info.red_bits = 32;
            info.green_bits = 32;
        }
        RenderFormat::D32FloatS8Uint => {
            info.depth_bits = 32;
            info.stencil_bits = 8;
            info.padding_bits = 24;
        }
        RenderFormat::R10g10b10a2Unorm | RenderFormat::R10g10b10a2Uint => {
            info.red_bits = 10;
            info.green_bits = 10;
            info.blue_bits = 10;
            info.alpha_bits = 2;
        }
        RenderFormat::R11g11b10Float => {
            info.red_bits = 11;
            info.green_bits = 11;
            info.blue_bits = 10;
        }
        RenderFormat::R8g8b8Unorm | RenderFormat::R8g8b8Srgb => {
            info.red_bits = 8;
            info.green_bits = 8;
            info.blue_bits = 8;
        }
        RenderFormat::R8g8b8a8Unorm
        | RenderFormat::R8g8b8a8Uint
        | RenderFormat::R8g8b8a8Snorm
        | RenderFormat::R8g8b8a8Sint => {
            info.red_bits = 8;
            info.green_bits = 8;
            info.blue_bits = 8;
            info.alpha_bits = 8;
        }
        RenderFormat::R16g16Float
        | RenderFormat::R16g16Unorm
        | RenderFormat::R16g16Uint
        | RenderFormat::R16g16Snorm
        | RenderFormat::R16g16Sint => {
            info.red_bits = 16;
            info.green_bits = 16;
        }
        RenderFormat::D32Float => {
            info.depth_bits = 32;
        }
        RenderFormat::R32Float | RenderFormat::R32Uint | RenderFormat::R32Sint => {
            info.red_bits = 32;
        }
        RenderFormat::D24UnormS8Uint => {
            info.depth_bits = 24;
            info.stencil_bits = 8;
        }
        RenderFormat::R8g8Unorm
        | RenderFormat::R8g8Uint
        | RenderFormat::R8g8Snorm
        | RenderFormat::R8g8Sint => {
            info.red_bits = 8;
            info.green_bits = 8;
        }
        RenderFormat::D16Unorm => {
            info.depth_bits = 16;
        }
        RenderFormat::R16Float
        | RenderFormat::R16Unorm
        | RenderFormat::R16Uint
        | RenderFormat::R16Snorm
        | RenderFormat::R16Sint => {
            info.red_bits = 16;
        }
        RenderFormat::R8Unorm
        | RenderFormat::R8Uint
        | RenderFormat::R8Snorm
        | RenderFormat::R8Sint => {
            info.red_bits = 8;
        }
        RenderFormat::R9g9b9e5Float => {
            info.red_bits = 9;
            info.green_bits = 9;
            info.blue_bits = 9;
            info.exponent_bits = 5;
        }
        RenderFormat::R5g6b5Unorm => {
            info.red_bits = 5;
            info.green_bits = 6;
            info.blue_bits = 5;
        }
        RenderFormat::R5g5b5a1Unorm => {
            info.red_bits = 5;
            info.green_bits = 6;
            info.blue_bits = 5;
            info.alpha_bits = 1;
        }
        RenderFormat::B8g8r8a8Unorm | RenderFormat::B8g8r8a8Srgb => {
            info.red_bits = 8;
            info.green_bits = 8;
            info.blue_bits = 8;
            info.alpha_bits = 8;
        }
        RenderFormat::Bc1Unorm
        | RenderFormat::Bc1aUnorm
        | RenderFormat::Bc2Unorm
        | RenderFormat::Bc3Unorm
        | RenderFormat::Bc4Unorm
        | RenderFormat::Bc4Snorm
        | RenderFormat::Bc5Unorm
        | RenderFormat::Bc5Snorm
        | RenderFormat::Bc1Srgb
        | RenderFormat::Bc1aSrgb
        | RenderFormat::Bc2Srgb
        | RenderFormat::Bc3Srgb
        | RenderFormat::Bc6uFloat
        | RenderFormat::Bc6sFloat
        | RenderFormat::Bc7Unorm
        | RenderFormat::Bc7Srgb => {
            // Ignore compressed formats
        }
        _ => {
            unimplemented!();
        }
    }

    // Calculate block size based on total bit count
    info.block_bits += info.red_bits;
    info.block_bits += info.green_bits;
    info.block_bits += info.blue_bits;
    info.block_bits += info.alpha_bits;
    info.block_bits += info.depth_bits;
    info.block_bits += info.stencil_bits;
    info.block_bits += info.padding_bits;
    info.block_bits += info.exponent_bits;

    // A block size of zero is a compressed format
    match info.block_bits {
        0 => match format {
            RenderFormat::Bc1Unorm
            | RenderFormat::Bc1Srgb
            | RenderFormat::Bc1aUnorm
            | RenderFormat::Bc1aSrgb
            | RenderFormat::Bc4Unorm => {
                info.block_bits = 64;
                info.block_width = 4;
                info.block_height = 4;
            }
            RenderFormat::Bc2Unorm
            | RenderFormat::Bc2Srgb
            | RenderFormat::Bc3Unorm
            | RenderFormat::Bc3Srgb
            | RenderFormat::Bc5Unorm
            | RenderFormat::Bc6uFloat
            | RenderFormat::Bc6sFloat
            | RenderFormat::Bc7Unorm
            | RenderFormat::Bc7Srgb => {
                info.block_bits = 128;
                info.block_width = 4;
                info.block_height = 4;
            }
            _ => {
                unimplemented!();
            }
        },
        _ => {
            info.block_width = 1;
            info.block_height = 1;
        }
    }

    info
}

#[inline(always)]
pub fn format_has_depth(format: RenderFormat) -> bool {
    match format {
        RenderFormat::D16Unorm
        | RenderFormat::D32Float
        | RenderFormat::D24UnormS8Uint
        | RenderFormat::D32FloatS8Uint => true,
        _ => false,
    }
}

#[inline(always)]
pub fn format_has_stencil(format: RenderFormat) -> bool {
    match format {
        RenderFormat::D24UnormS8Uint | RenderFormat::D32FloatS8Uint => true,
        _ => false,
    }
}

#[inline(always)]
pub fn channel_format_has_stencil(format: RenderChannelFormat) -> bool {
    match format {
        RenderChannelFormat::D24S8 | RenderChannelFormat::D32S8 => true,
        _ => false,
    }
}

#[inline(always)]
pub fn channel_format_has_depth(format: RenderChannelFormat) -> bool {
    match format {
        RenderChannelFormat::D16
        | RenderChannelFormat::D24S8
        | RenderChannelFormat::D24
        | RenderChannelFormat::D32
        | RenderChannelFormat::D32S8 => true,
        _ => false,
    }
}

#[inline(always)]
pub fn channel_format_is_dxt(format: RenderChannelFormat) -> bool {
    match format {
        RenderChannelFormat::BC1
        | RenderChannelFormat::BC1A
        | RenderChannelFormat::BC2
        | RenderChannelFormat::BC3
        | RenderChannelFormat::BC4
        | RenderChannelFormat::BC5
        | RenderChannelFormat::BC6U
        | RenderChannelFormat::BC6S
        | RenderChannelFormat::BC7 => true,
        _ => false,
    }
}

#[inline(always)]
pub fn channel_format_is_astc(format: RenderChannelFormat) -> bool {
    match format {
        RenderChannelFormat::Astc4x4
        | RenderChannelFormat::Astc5x4
        | RenderChannelFormat::Astc5x5
        | RenderChannelFormat::Astc6x5
        | RenderChannelFormat::Astc6x6
        | RenderChannelFormat::Astc8x5
        | RenderChannelFormat::Astc8x6
        | RenderChannelFormat::Astc8x8
        | RenderChannelFormat::Astc10x5
        | RenderChannelFormat::Astc10x6
        | RenderChannelFormat::Astc10x8
        | RenderChannelFormat::Astc10x10
        | RenderChannelFormat::Astc12x10
        | RenderChannelFormat::Astc12x12 => true,
        _ => false,
    }
}

#[inline(always)]
pub fn channel_format_is_compressed(format: RenderChannelFormat) -> bool {
    channel_format_is_dxt(format) || channel_format_is_astc(format)
}

#[inline(always)]
pub fn channel_format_has_alpha(format: RenderChannelFormat) -> bool {
    match channel_format_component_count(format) {
        4 => true,
        _ => false,
    }
}

#[inline(always)]
pub fn channel_format_component_count(format: RenderChannelFormat) -> u32 {
    match format {
        RenderChannelFormat::Unknown => 0,

        RenderChannelFormat::BC4
        | RenderChannelFormat::R8
        | RenderChannelFormat::R16
        | RenderChannelFormat::R32
        | RenderChannelFormat::D16
        | RenderChannelFormat::D24
        | RenderChannelFormat::D32 => 1,

        RenderChannelFormat::BC5
        | RenderChannelFormat::R4G4
        | RenderChannelFormat::R8G8
        | RenderChannelFormat::R16G16
        | RenderChannelFormat::R32G32
        | RenderChannelFormat::D24S8
        | RenderChannelFormat::D32S8 => 2,

        RenderChannelFormat::R5G6B5
        | RenderChannelFormat::R8G8B8
        | RenderChannelFormat::R10G11B11
        | RenderChannelFormat::R11G11B10
        | RenderChannelFormat::R9G9B9E5
        | RenderChannelFormat::R32G32B32
        | RenderChannelFormat::BC6U
        | RenderChannelFormat::BC6S => 3,

        RenderChannelFormat::R5G5B5A1
        | RenderChannelFormat::R4G4B4A4
        | RenderChannelFormat::R8G8B8A8
        | RenderChannelFormat::B8G8R8A8
        | RenderChannelFormat::R16G16B16A16
        | RenderChannelFormat::R32G32B32A32
        | RenderChannelFormat::BC1
        | RenderChannelFormat::BC1A
        | RenderChannelFormat::BC2
        | RenderChannelFormat::BC3
        | RenderChannelFormat::BC7
        | RenderChannelFormat::R10G10B10A2
        | RenderChannelFormat::Astc4x4
        | RenderChannelFormat::Astc5x4
        | RenderChannelFormat::Astc5x5
        | RenderChannelFormat::Astc6x5
        | RenderChannelFormat::Astc6x6
        | RenderChannelFormat::Astc8x5
        | RenderChannelFormat::Astc8x6
        | RenderChannelFormat::Astc8x8
        | RenderChannelFormat::Astc10x5
        | RenderChannelFormat::Astc10x6
        | RenderChannelFormat::Astc10x8
        | RenderChannelFormat::Astc10x10
        | RenderChannelFormat::Astc12x10
        | RenderChannelFormat::Astc12x12 => 4,
    }
}

#[inline(always)]
pub fn channel_format_bit_count(format: RenderChannelFormat) -> u32 {
    match format {
        RenderChannelFormat::Astc12x12 => 1,

        RenderChannelFormat::Astc12x10
        | RenderChannelFormat::Astc10x10
        | RenderChannelFormat::Astc10x8
        | RenderChannelFormat::Astc8x8 => 2,

        RenderChannelFormat::Astc10x6
        | RenderChannelFormat::Astc10x5
        | RenderChannelFormat::Astc8x6 => 3,

        RenderChannelFormat::BC1
        | RenderChannelFormat::BC1A
        | RenderChannelFormat::BC4
        | RenderChannelFormat::Astc8x5
        | RenderChannelFormat::Astc6x6 => 4,

        RenderChannelFormat::Astc6x5 => 5,

        RenderChannelFormat::Astc5x5 => 6,

        RenderChannelFormat::Astc5x4 => 7,

        RenderChannelFormat::R4G4
        | RenderChannelFormat::R8
        | RenderChannelFormat::BC2
        | RenderChannelFormat::BC3
        | RenderChannelFormat::BC5
        | RenderChannelFormat::BC6U
        | RenderChannelFormat::BC6S
        | RenderChannelFormat::BC7
        | RenderChannelFormat::Astc4x4 => 8,

        RenderChannelFormat::R8G8
        | RenderChannelFormat::R5G6B5
        | RenderChannelFormat::R5G5B5A1
        | RenderChannelFormat::R4G4B4A4
        | RenderChannelFormat::R16
        | RenderChannelFormat::D16 => 16,

        RenderChannelFormat::D24 => 24,

        RenderChannelFormat::R8G8B8A8
        | RenderChannelFormat::B8G8R8A8
        | RenderChannelFormat::R32
        | RenderChannelFormat::D24S8
        | RenderChannelFormat::D32
        | RenderChannelFormat::R16G16
        | RenderChannelFormat::R10G10B10A2
        | RenderChannelFormat::R11G11B10
        | RenderChannelFormat::R9G9B9E5 => 32,

        RenderChannelFormat::R16G16B16A16
        | RenderChannelFormat::R32G32
        | RenderChannelFormat::D32S8 => 64,

        RenderChannelFormat::R32G32B32A32 => 128,

        _ => {
            unimplemented!();
        }
    }
}

#[inline(always)]
pub fn vertex_count_from_primitive_count(
    primitive_count: u32,
    primitive_type: RenderPrimitiveType,
) -> u32 {
    match primitive_type {
        RenderPrimitiveType::PointList => primitive_count,
        RenderPrimitiveType::LineList => primitive_count * 2,
        RenderPrimitiveType::LineStrip => primitive_count + 1,
        RenderPrimitiveType::TriangleList => primitive_count * 3,
        RenderPrimitiveType::TriangleStrip => primitive_count + 2,
        RenderPrimitiveType::QuadList => primitive_count * 4,
        RenderPrimitiveType::RectList => primitive_count * 3,
        RenderPrimitiveType::TrianglePatch => primitive_count * 3,
    }
}

#[inline(always)]
pub fn primitive_count_from_vertex_count(
    vertex_count: u32,
    primitive_type: RenderPrimitiveType,
) -> u32 {
    match primitive_type {
        RenderPrimitiveType::PointList => vertex_count,
        RenderPrimitiveType::LineList => vertex_count / 2,
        RenderPrimitiveType::LineStrip => {
            assert_ne!(vertex_count, 1);
            if vertex_count == 0 {
                0
            } else {
                vertex_count - 1
            }
        }
        RenderPrimitiveType::TriangleList => vertex_count / 3,
        RenderPrimitiveType::TriangleStrip => {
            assert_ne!(vertex_count, 1);
            assert_ne!(vertex_count, 2);
            if vertex_count <= 1 {
                0
            } else {
                vertex_count - 2
            }
        }
        RenderPrimitiveType::QuadList => vertex_count / 4,
        RenderPrimitiveType::RectList => vertex_count / 3,
        RenderPrimitiveType::TrianglePatch => vertex_count / 3,
    }
}

#[inline(always)]
pub fn channel_format_min_dimensions(format: RenderChannelFormat) -> (u32, u32) {
    match channel_format_is_dxt(format) {
        true => (4, 4),
        false => match format {
            RenderChannelFormat::Astc4x4 => (4, 4),
            RenderChannelFormat::Astc5x4 => (5, 4),
            RenderChannelFormat::Astc5x5 => (5, 5),
            RenderChannelFormat::Astc6x5 => (6, 5),
            RenderChannelFormat::Astc6x6 => (6, 6),
            RenderChannelFormat::Astc8x5 => (8, 5),
            RenderChannelFormat::Astc8x6 => (8, 6),
            RenderChannelFormat::Astc8x8 => (8, 8),
            RenderChannelFormat::Astc10x5 => (10, 5),
            RenderChannelFormat::Astc10x6 => (10, 6),
            RenderChannelFormat::Astc10x8 => (10, 8),
            RenderChannelFormat::Astc10x10 => (10, 10),
            RenderChannelFormat::Astc12x10 => (12, 10),
            RenderChannelFormat::Astc12x12 => (12, 12),
            _ => (1, 1),
        },
    }
}

#[inline(always)]
pub fn texture_view_dimension(texture_type: RenderTextureType) -> RenderViewDimension {
    match texture_type {
        RenderTextureType::Tex1d => RenderViewDimension::Tex1d,
        RenderTextureType::Tex1dArray => RenderViewDimension::Tex1dArray,
        RenderTextureType::Tex2d => RenderViewDimension::Tex2d,
        RenderTextureType::Tex2dArray => RenderViewDimension::Tex2dArray,
        RenderTextureType::Tex3d => RenderViewDimension::Tex3d,
        RenderTextureType::Cube => RenderViewDimension::Cube,
        RenderTextureType::CubeArray => RenderViewDimension::CubeArray,
    }
}

#[inline(always)]
pub fn get_texture_layout_info(
    format: RenderFormat,
    width: u32,
    height: u32,
) -> RenderTextureLayoutInfo {
    use std::cmp::max;
    let format_info = get_render_format_info(format);
    let width_by_block = max(1, width / format_info.block_width);
    let height_by_block = max(1, height / format_info.block_height);
    RenderTextureLayoutInfo {
        pitch: (width_by_block * format_info.block_bits) / 8,
        slice_pitch: (width_by_block * height_by_block * format_info.block_bits) / 8,
    }
}

#[inline(always)]
pub fn get_texture_size(
    format: RenderFormat,
    width: u32,
    height: u32,
    depth: u32,
    levels: u32,
    elements: u32,
) -> usize {
    use std::cmp::max;

    let mut size: usize = 0;

    let mut width = width;
    let mut height = height;
    let mut depth = depth;

    let format_info = get_render_format_info(format);
    for _ in 0..levels {
        let blocks_w = align_forward(width as usize, format_info.block_width as usize)
            / format_info.block_width as usize;
        let blocks_h = align_forward(height as usize, format_info.block_height as usize)
            / format_info.block_height as usize;
        let blocks_d = depth as usize;
        size += (format_info.block_bits as usize * blocks_w * blocks_h * blocks_d) / 8usize;
        width = max(width >> 2, 1);
        height = max(height >> 2, 1);
        depth = max(depth >> 2, 1);
    }

    size * elements as usize
}

pub fn build_resource_state_text(states: RenderResourceStates) -> String {
    let mut result = String::with_capacity(1046);

    if states == RenderResourceStates::COMMON {
        result.push_str("Common");
    }

    if states.contains(RenderResourceStates::RENDER_TARGET) {
        result.push_str("RenderTarget|");
    }

    if states.contains(RenderResourceStates::UNORDERED_ACCESS) {
        result.push_str("UnorderedAccess|");
    }

    if states.contains(RenderResourceStates::DEPTH_WRITE) {
        result.push_str("DepthWrite|");
    }

    if states.contains(RenderResourceStates::DEPTH_READ) {
        result.push_str("DepthRead|");
    }

    if states.contains(RenderResourceStates::STREAM_OUT) {
        result.push_str("StreamOut|");
    }

    if states.contains(RenderResourceStates::COPY_DEST) {
        result.push_str("CopyDest|");
    }

    if states.contains(RenderResourceStates::RESOLVE_DEST) {
        result.push_str("ResolveDest|");
    }

    if states.contains(RenderResourceStates::RESOLVE_SOURCE) {
        result.push_str("ResolveSource|");
    }

    if states == RenderResourceStates::GENERIC_READ {
        result.push_str("GenericRead|");
    } else {
        if states.contains(RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER) {
            result.push_str("VertexAndConstantBuffer|");
        }

        if states.contains(RenderResourceStates::INDEX_BUFFER) {
            result.push_str("IndexBuffer|");
        }

        if states.contains(RenderResourceStates::COPY_SOURCE) {
            result.push_str("CopySource|");
        }

        if states.contains(RenderResourceStates::NON_PIXEL_SHADER_RESOURCE) {
            result.push_str("NonPixelShaderResource|");
        }

        if states.contains(RenderResourceStates::PIXEL_SHADER_RESOURCE) {
            result.push_str("PixelShaderResource|");
        }

        if states.contains(RenderResourceStates::INDIRECT_ARGUMENT) {
            result.push_str("IndirectArgument|");
        }
    }

    result
}
