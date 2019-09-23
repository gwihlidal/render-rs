use crate::constants::*;
use crate::handles::RenderResourceHandle;
use crate::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderSamplerFilter {
    MinMagMipPoint = 0,
    MinMagPointMipLinear = 1,
    MinPointMagLinearMipPoint = 2,
    MinPointMagMipLinear = 3,
    MinLinearMagMipPoint = 4,
    MinLinearMagPointMipLinear = 5,
    MinMagLinearMipPoint = 6,
    MinMagMipLinear = 7,
    Anisotropic = 8,
    ComparisonMinMagMipPoint = 9,
    ComparisonMinMagPointMipLinear = 10,
    ComparisonMinPointMagLinearMipPoint = 11,
    ComparisonMinPointMagMipLinear = 12,
    ComparisonMinLinearMagMipPoint = 13,
    ComparisonMinLinearMagPointMipLinear = 14,
    ComparisonMinMagLinearMipPoint = 15,
    ComparisonMinMagMipLinear = 16,
    ComparisonAnisotropic = 17,
    MinimumMinMagMipPoint = 18,
    MinimumMinMagPointMipLinear = 19,
    MinimumMinPointMagLinearMipPoint = 20,
    MinimumMinPointMagMipLinear = 21,
    MinimumMinLinearMagMipPoint = 22,
    MinimumMinLinearMagPointMipLinear = 23,
    MinimumMinMagLinearMipPoint = 24,
    MinimumMinMagMipLinear = 25,
    MinimumAnisotropic = 26,
    MaximumMinMagMipPoint = 27,
    MaximumMinMagPointMipLinear = 28,
    MaximumMinPointMagLinearMipPoint = 29,
    MaximumMinPointMagMipLinear = 30,
    MaximumMinLinearMagMipPoint = 31,
    MaximumMinLinearMagPointMipLinear = 32,
    MaximumMinMagLinearMipPoint = 33,
    MaximumMinMagMipLinear = 34,
    MaximumAnisotropic = 35,
}

// TODO: Make more idiomatic rust
pub fn is_comparison_filter(filter: RenderSamplerFilter) -> bool {
    match filter {
        RenderSamplerFilter::ComparisonMinMagMipPoint
        | RenderSamplerFilter::ComparisonMinMagPointMipLinear
        | RenderSamplerFilter::ComparisonMinPointMagLinearMipPoint
        | RenderSamplerFilter::ComparisonMinPointMagMipLinear
        | RenderSamplerFilter::ComparisonMinLinearMagMipPoint
        | RenderSamplerFilter::ComparisonMinLinearMagPointMipLinear
        | RenderSamplerFilter::ComparisonMinMagLinearMipPoint
        | RenderSamplerFilter::ComparisonMinMagMipLinear
        | RenderSamplerFilter::ComparisonAnisotropic => true,
        _ => false,
    }
}

pub fn is_min_filter(filter: RenderSamplerFilter) -> bool {
    match filter {
        RenderSamplerFilter::MinimumMinMagMipPoint
        | RenderSamplerFilter::MinimumMinMagPointMipLinear
        | RenderSamplerFilter::MinimumMinPointMagLinearMipPoint
        | RenderSamplerFilter::MinimumMinPointMagMipLinear
        | RenderSamplerFilter::MinimumMinLinearMagMipPoint
        | RenderSamplerFilter::MinimumMinLinearMagPointMipLinear
        | RenderSamplerFilter::MinimumMinMagLinearMipPoint
        | RenderSamplerFilter::MinimumMinMagMipLinear
        | RenderSamplerFilter::MinimumAnisotropic => true,
        _ => false,
    }
}

pub fn is_max_filter(filter: RenderSamplerFilter) -> bool {
    match filter {
        RenderSamplerFilter::MaximumMinMagMipPoint
        | RenderSamplerFilter::MaximumMinMagPointMipLinear
        | RenderSamplerFilter::MaximumMinPointMagLinearMipPoint
        | RenderSamplerFilter::MaximumMinPointMagMipLinear
        | RenderSamplerFilter::MaximumMinLinearMagMipPoint
        | RenderSamplerFilter::MaximumMinLinearMagPointMipLinear
        | RenderSamplerFilter::MaximumMinMagLinearMipPoint
        | RenderSamplerFilter::MaximumMinMagMipLinear
        | RenderSamplerFilter::MaximumAnisotropic => true,
        _ => false,
    }
}

pub fn is_min_max_filter(filter: RenderSamplerFilter) -> bool {
    is_min_filter(filter) || is_max_filter(filter)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderBorderColor {
    /// Transparent black (0,0,0,0)
    BlackA0 = 0,

    /// Opaque black (0,0,0,1)
    BlackA1 = 1,

    /// Opaque white (1,1,1,1)
    WhiteA1 = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderCullMode {
    None = 0,
    Front = 1,
    Back = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderBlendMode {
    Zero = 0,
    One = 1,
    SourceColor = 2,
    InvSourceColor = 3,
    SourceAlpha = 4,
    InvSourceAlpha = 5,
    DestColor = 6,
    InvDestColor = 7,
    DestAlpha = 8,
    InvDestAlpha = 9,
    SourceAlphaSaturate = 10,
    Constant = 11,
    InvConstant = 12,
    Source1Color = 13,
    InvSource1Color = 14,
    Source1Alpha = 15,
    InvSource1Alpha = 16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderBlendOp {
    /// Add source 1 and source 2.
    Add = 0,

    /// Subtract source 1 from source 2.
    Subtract = 1,

    /// Subtract source 2 from source 1.
    RevSubtract = 2,

    /// Find the minimum of source 1 and source 2.
    Min = 3,

    /// Find the maximum of source 1 and source 2.
    Max = 4,
}

bitflags! {
    pub struct RenderWriteMask: u8 {
        const NONE = 0;
        const RED = 0x1;
        const GREEN = 0x2;
        const BLUE = 0x4;
        const ALPHA = 0x8;
        const COLOR = 0x1 | 0x2 | 0x4;
        const ALL = 0x1 | 0x2 | 0x4 | 0x8;
    }
}

impl Default for RenderWriteMask {
    fn default() -> Self {
        RenderWriteMask::ALL
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderFillMode {
    Solid = 0,
    WireFrame = 1,
}

#[derive(Clone, Default, Debug)]
pub struct RenderGraphicsPipelineStateDesc {
    pub shaders: [RenderResourceHandle; MAX_SHADER_TYPE],
    pub shader_signature: RenderShaderSignatureDesc,
    pub render_state: RenderState,
    pub vertex_element_count: u32,
    pub vertex_elements: [RenderVertexElement; MAX_VERTEX_ELEMENTS],
    pub vertex_buffer_strides: [u32; MAX_VERTEX_ELEMENTS],
    pub primitive_type: RenderPrimitiveType,
    pub render_target_count: u32,
    pub render_target_write_masks: [RenderWriteMask; MAX_RENDER_TARGET_COUNT],
    pub render_target_formats: [RenderFormat; MAX_RENDER_TARGET_COUNT],
    pub depth_stencil_format: RenderFormat,
}

#[derive(Clone, Debug)]
pub struct RenderComputePipelineStateDesc {
    pub shader: RenderResourceHandle,
    pub shader_signature: RenderShaderSignatureDesc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderClearMask {
    Color0 = 1,
    Color1 = 2,
    Color2 = 4,
    Color3 = 8,
    Color4 = 16,
    Color5 = 32,
    Color6 = 64,
    Color7 = 128,
    Depth = 256,
    Stencil = 512,

    Color = 255,
    All = 1023, // Color|Depth|Stencil
    None = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderCompareFunc {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderStencilOp {
    Keep = 0,
    Zero = 1,
    Replace = 2,
    IncrementSaturate = 3,
    DecrementSaturate = 4,
    Invert = 5,
    IncrementWrap = 6,
    DecrementWrap = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderSamplerAddressMode {
    Wrap = 0,
    Mirror = 1,
    Clamp = 2,
    Border = 3,
    MirrorOnce = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderStencilMode {
    /// Stenciling is disabled
    Disabled = 0,

    /// Single-sided stencil test, back-side test is undefined if surface is rendered double-sided
    SingleSided = 1,

    /// Double-sided stencil test with individual settings for each side (front & back)
    DoubleSided = 2,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderStencilSide {
    pub func: RenderCompareFunc,
    pub fail_op: RenderStencilOp,
    pub depth_fail_op: RenderStencilOp,
    pub pass_op: RenderStencilOp,
}

impl Default for RenderStencilSide {
    fn default() -> Self {
        Self {
            func: RenderCompareFunc::Always,
            fail_op: RenderStencilOp::Keep,
            depth_fail_op: RenderStencilOp::Keep,
            pass_op: RenderStencilOp::Keep,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderStencilState {
    pub mode: RenderStencilMode,
    pub read_mask: u8,
    pub write_mask: u8,
    pub front: RenderStencilSide,
    pub back: RenderStencilSide,
}

impl Default for RenderStencilState {
    fn default() -> Self {
        Self {
            mode: RenderStencilMode::Disabled,
            read_mask: ::std::u8::MAX,
            write_mask: ::std::u8::MAX,
            front: Default::default(),
            back: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct RenderDrawState {
    pub viewport: Option<RenderViewportRect>,
    pub scissor: Option<RenderScissorRect>,
    pub stencil_ref: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct RenderViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_z: f32,
    pub max_z: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd)]
pub struct RenderScissorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct RenderSamplerState {
    pub filter: RenderSamplerFilter,
    pub address_u: RenderSamplerAddressMode,
    pub address_v: RenderSamplerAddressMode,
    pub address_w: RenderSamplerAddressMode,
    pub comparison: RenderCompareFunc,
    pub border_color: RenderBorderColor,
    /// Selects the minimum mipmap index to use. Max value is 15
    pub min_mip: u16,
    /// Selects the maximum mipmap index to use. Max value is 15
    pub max_mip: u16,
    /// Max level of anisotropic filtering. Max value is 16
    pub max_aniso: u16,
}

impl Default for RenderSamplerState {
    fn default() -> Self {
        Self {
            filter: RenderSamplerFilter::MinMagMipLinear,
            address_u: RenderSamplerAddressMode::Wrap,
            address_v: RenderSamplerAddressMode::Wrap,
            address_w: RenderSamplerAddressMode::Wrap,
            comparison: RenderCompareFunc::Always,
            border_color: RenderBorderColor::BlackA0,
            min_mip: 0,
            max_mip: 15,
            max_aniso: 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderBlendState {
    pub source_color: RenderBlendMode,
    pub source_alpha: RenderBlendMode,
    pub dest_color: RenderBlendMode,
    pub dest_alpha: RenderBlendMode,
    pub op_color: RenderBlendOp,
    pub op_alpha: RenderBlendOp,
    pub blend_enable: bool,
}

impl Default for RenderBlendState {
    fn default() -> Self {
        Self {
            source_color: RenderBlendMode::One,
            source_alpha: RenderBlendMode::One,
            dest_color: RenderBlendMode::Zero,
            dest_alpha: RenderBlendMode::Zero,
            op_color: RenderBlendOp::Add,
            op_alpha: RenderBlendOp::Add,
            blend_enable: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderState {
    pub blend_states: [RenderBlendState; MAX_RENDER_TARGET_COUNT],
    pub stencil: RenderStencilState,
    pub depth_enable: bool, // TODO: Make a generic D3D12_DEPTH_WRITE_MASK
    pub depth_clamp: bool,
    pub depth_write_mask: u32,
    pub depth_bias: f32,
    pub slope_scaled_depth_bias: f32,
    pub depth_func: RenderCompareFunc,
    pub fill_mode: RenderFillMode,
    pub cull_mode: RenderCullMode,
    pub anti_aliased_line_enabled: bool, // TODO: Remove this?
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            blend_states: Default::default(),
            stencil: Default::default(),
            depth_enable: false,
            depth_clamp: false,
            depth_write_mask: 0,
            depth_bias: 0f32,
            slope_scaled_depth_bias: 0f32,
            depth_func: RenderCompareFunc::GreaterEqual,
            fill_mode: RenderFillMode::Solid,
            cull_mode: RenderCullMode::None,
            anti_aliased_line_enabled: false,
        }
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u8)]
    pub enum RenderVertexUsage {
        Unknown = 0,
        Position = 1,
        BlendWeights = 2,
        BlendIndices = 3,
        Normal = 4,
        TexCoord = 5,
        Tangent = 6,
        Bitangent = 7,
        Color = 8,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderVertexElement {
    pub stream: u32,
    pub offset: u32,
    pub format: RenderFormat,
    pub usage: RenderVertexUsage,
    pub usage_index: u32,
}

impl Default for RenderVertexElement {
    fn default() -> Self {
        use std::u32::MAX;
        Self {
            stream: MAX,
            offset: MAX,
            format: RenderFormat::Unknown,
            usage: RenderVertexUsage::Unknown,
            usage_index: MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBindingBuffer {
    pub resource: RenderResourceHandle,
    pub offset: usize,
    pub size: usize,
    pub stride: u32,
}

pub type RenderBindingConstantBuffer = RenderBindingBuffer;

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBindingView {
    pub resource: RenderResourceHandle,
    pub format: RenderFormat,
    pub dimension: RenderViewDimension,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBindingRenderTargetView {
    pub base: RenderBindingView,
    pub mip_slice: u32,
    pub first_array_slice: u32,
    pub plane_slice_first_w_slice: u32, // TODO: Use either-or enum
    pub array_size: u32,
    pub w_size: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBindingDepthStencilView {
    pub base: RenderBindingView,
    pub flags: RenderDepthStencilViewFlags,
    pub mip_slice: u32,
    pub first_array_slice: u32,
    pub array_size: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBindingShaderResourceView {
    pub base: RenderBindingView,
    pub most_detailed_mip_first_element: u32, // TODO: Use either-or enum
    pub mip_levels_element_count: u32,        // TODO: Use either-or enum
    pub first_array_slice: u32,
    pub plane_slice: u32,
    pub array_size: u32,
    pub struct_byte_stride: u32,
    pub resource_min_lod_clamp: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBindingUnorderedAccessView {
    pub base: RenderBindingView,
    pub mip_slice_first_element: u32, // TODO: Use either-or enum
    pub first_array_slice_first_w_slice_element_count: u32, // TODO: Use either-or enum
    pub array_size_plane_slice_w_size: u32, // TODO: Use either-or enum
    pub struct_byte_stride: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderShaderTableEntry {
    pub program: RenderResourceHandle,
    pub shader_arguments: Vec<RenderShaderArgument>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderShaderTableUpdateDesc {
    // Must be less than or equal to the counts set at creation time
    pub ray_gen_entries: Vec<RenderShaderTableEntry>,
    pub hit_entries: Vec<RenderShaderTableEntry>,
    pub miss_entries: Vec<RenderShaderTableEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderAccelerationPerformanceHint {
    Balanced,
    PreferFastTrace,
    PreferFastBuild,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderAccelerationBottomDesc {
    pub geometry: RenderResourceHandle,
    pub acceleration_buffer: RenderResourceHandle,
    pub acceleration_scratch: RenderResourceHandle,
    pub performance_hint: RenderAccelerationPerformanceHint,
    pub allow_refit: bool,
}

impl Default for RenderAccelerationBottomDesc {
    fn default() -> Self {
        use std::u32::MAX;
        Self {
            geometry: Default::default(),
            acceleration_buffer: Default::default(),
            acceleration_scratch: Default::default(),
            performance_hint: RenderAccelerationPerformanceHint::Balanced,
            allow_refit: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderAccelerationInstance {
    pub transform: [f32; 12],
    pub id: u32,
    pub contribution_to_hit_group_index: u32,
    pub flags: u32,
    pub acceleration_buffer: RenderResourceHandle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderAccelerationTopDesc {
    pub instances: Vec<RenderAccelerationInstance>,
    pub instances_buffer: RenderResourceHandle,
    pub acceleration_buffer: RenderResourceHandle,
    pub acceleration_scratch: RenderResourceHandle,
    pub performance_hint: RenderAccelerationPerformanceHint,
}

impl Default for RenderAccelerationTopDesc {
    fn default() -> Self {
        Self {
            instances: Default::default(),
            instances_buffer: Default::default(),
            acceleration_buffer: Default::default(),
            acceleration_scratch: Default::default(),
            performance_hint: RenderAccelerationPerformanceHint::Balanced,
        }
    }
}

pub mod build {
    use super::*;

    #[inline(always)]
    pub fn ray_tracing_acceleration(
        resource: RenderResourceHandle,
    ) -> RenderBindingShaderResourceView {
        //assert(!resource || RenderSystem::getHandleAllocator().isValid(resource));
        //assert!(resource.is_valid());
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format: RenderFormat::Unknown,
                dimension: RenderViewDimension::TopLevelAccelerationStructure,
            },
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn constant_buffer(
        resource: RenderResourceHandle,
        size: usize,
        offset: usize,
    ) -> RenderBindingConstantBuffer {
        assert_ne!(size, 0);
        RenderBindingConstantBuffer {
            resource,
            offset,
            size,
            stride: size as u32, // in most cases, the size == stride
        }
    }

    #[inline(always)]
    pub fn buffer(
        resource: RenderResourceHandle,
        format: RenderFormat,
        first_element: u32,
        element_count: u32,
        struct_byte_stride: u32,
    ) -> RenderBindingShaderResourceView {
        assert_ne!(element_count, 0);
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Buffer,
            },
            most_detailed_mip_first_element: first_element,
            mip_levels_element_count: element_count,
            struct_byte_stride,
            first_array_slice: 0,
            array_size: 0,
            plane_slice: 0,
            resource_min_lod_clamp: 0f32,
        }
    }

    #[inline(always)]
    pub fn texture_1d(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex1d,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            resource_min_lod_clamp: min_lod_clamp,
            struct_byte_stride: 0,
            array_size: 0,
            first_array_slice: 0,
            plane_slice: 0,
        }
    }

    #[inline(always)]
    pub fn texture_1d_array(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        first_array_slice: u32,
        array_size: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex1dArray,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            resource_min_lod_clamp: min_lod_clamp,
            first_array_slice,
            array_size,
            struct_byte_stride: 0,
            plane_slice: 0,
        }
    }

    #[inline(always)]
    pub fn texture_2d(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        plane_slice: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex2d,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            plane_slice,
            resource_min_lod_clamp: min_lod_clamp,
            array_size: 0,
            first_array_slice: 0,
            struct_byte_stride: 0,
        }
    }

    #[inline(always)]
    pub fn texture_2d_array(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        first_array_slice: u32,
        array_size: u32,
        plane_slice: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex2dArray,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            first_array_slice,
            array_size,
            plane_slice,
            resource_min_lod_clamp: min_lod_clamp,
            struct_byte_stride: 0,
        }
    }

    #[inline(always)]
    pub fn texture_3d(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex3d,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            first_array_slice: 0,
            array_size: 0,
            plane_slice: 0,
            resource_min_lod_clamp: min_lod_clamp,
            struct_byte_stride: 0,
        }
    }

    #[inline(always)]
    pub fn texture_cube(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Cube,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            first_array_slice: 0,
            array_size: 0,
            plane_slice: 0,
            resource_min_lod_clamp: min_lod_clamp,
            struct_byte_stride: 0,
        }
    }

    #[inline(always)]
    pub fn texture_cube_array(
        resource: RenderResourceHandle,
        format: RenderFormat,
        most_detailed_mip: u32,
        mip_levels: u32,
        first_2d_array_face: u32,
        cube_count: u32,
        min_lod_clamp: f32,
    ) -> RenderBindingShaderResourceView {
        RenderBindingShaderResourceView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::CubeArray,
            },
            most_detailed_mip_first_element: most_detailed_mip,
            mip_levels_element_count: mip_levels,
            first_array_slice: first_2d_array_face,
            array_size: cube_count,
            plane_slice: 0,
            resource_min_lod_clamp: min_lod_clamp,
            struct_byte_stride: 0,
        }
    }

    #[inline(always)]
    pub fn buffer_rw(
        resource: RenderResourceHandle,
        format: RenderFormat,
        first_element: u32,
        element_count: u32,
        struct_byte_stride: u32,
    ) -> RenderBindingUnorderedAccessView {
        assert_ne!(element_count, 0);
        RenderBindingUnorderedAccessView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Buffer,
            },
            mip_slice_first_element: first_element,
            first_array_slice_first_w_slice_element_count: element_count,
            struct_byte_stride,
            array_size_plane_slice_w_size: 0,
        }
    }

    #[inline(always)]
    pub fn texture_1d_rw(
        resource: RenderResourceHandle,
        format: RenderFormat,
        mip_slice: u32,
    ) -> RenderBindingUnorderedAccessView {
        RenderBindingUnorderedAccessView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex1d,
            },
            mip_slice_first_element: mip_slice,
            first_array_slice_first_w_slice_element_count: 0,
            struct_byte_stride: 0,
            array_size_plane_slice_w_size: 0,
        }
    }

    #[inline(always)]
    pub fn texture_1d_array_rw(
        resource: RenderResourceHandle,
        format: RenderFormat,
        mip_slice: u32,
        first_array_slice: u32,
        array_size: u32,
    ) -> RenderBindingUnorderedAccessView {
        RenderBindingUnorderedAccessView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex1dArray,
            },
            mip_slice_first_element: mip_slice,
            first_array_slice_first_w_slice_element_count: first_array_slice,
            struct_byte_stride: 0,
            array_size_plane_slice_w_size: array_size,
        }
    }

    #[inline(always)]
    pub fn texture_2d_rw(
        resource: RenderResourceHandle,
        format: RenderFormat,
        mip_slice: u32,
        plane_slice: u32,
    ) -> RenderBindingUnorderedAccessView {
        RenderBindingUnorderedAccessView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex2d,
            },
            mip_slice_first_element: mip_slice,
            first_array_slice_first_w_slice_element_count: 0,
            struct_byte_stride: 0,
            array_size_plane_slice_w_size: plane_slice,
        }
    }

    #[inline(always)]
    pub fn texture_2d_array_rw(
        resource: RenderResourceHandle,
        format: RenderFormat,
        mip_slice: u32,
        _plane_slice: u32,
        first_array_slice: u32,
        array_size: u32,
    ) -> RenderBindingUnorderedAccessView {
        RenderBindingUnorderedAccessView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex2dArray,
            },
            mip_slice_first_element: mip_slice,
            first_array_slice_first_w_slice_element_count: first_array_slice,
            struct_byte_stride: 0,
            array_size_plane_slice_w_size: array_size,
        }
    }

    #[inline(always)]
    pub fn texture_3d_rw(
        resource: RenderResourceHandle,
        format: RenderFormat,
        mip_slice: u32,
        first_w_slice: u32,
        w_size: u32,
    ) -> RenderBindingUnorderedAccessView {
        RenderBindingUnorderedAccessView {
            base: RenderBindingView {
                resource,
                format,
                dimension: RenderViewDimension::Tex3d,
            },
            mip_slice_first_element: mip_slice,
            first_array_slice_first_w_slice_element_count: first_w_slice,
            struct_byte_stride: 0,
            array_size_plane_slice_w_size: w_size,
        }
    }
}
