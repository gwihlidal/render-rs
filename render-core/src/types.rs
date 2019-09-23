#![allow(dead_code)]

use crate::constants::*;
pub use crate::handles::RenderResourceHandle;
use crate::state::*;
use enum_count::EnumCount;
use std::cmp;
use std::slice::Iter;
use winit;

pub type RenderTransitionRecord = (RenderResourceHandle, RenderResourceStates);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderQueueType {
    Universal = 0,
    Compute = 1,
    Transfer = 2,
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u32)]
    pub enum RenderFormat {
        Unknown = 0,
        R4g4Unorm = 1,
        R4g4b4a4Unorm = 2,
        R5g6b5Unorm = 3,
        R5g5b5a1Unorm = 4,
        R8Unorm = 5,
        R8Snorm = 6,
        R8Srgb = 7,
        R8Uint = 8,
        R8Sint = 9,
        R8g8Unorm = 10,
        R8g8Snorm = 11,
        R8g8Srgb = 12,
        R8g8Uint = 13,
        R8g8Sint = 14,
        R8g8b8Unorm = 15,
        R8g8b8Srgb = 16,
        R8g8b8a8Unorm = 17,
        R8g8b8a8Snorm = 18,
        R8g8b8a8Srgb = 19,
        R8g8b8a8Uint = 20,
        R8g8b8a8Sint = 21,
        B8g8r8a8Unorm = 22,
        B8g8r8a8Srgb = 23,
        R11g11b10Float = 24,
        R10g10b10a2Unorm = 25,
        R10g10b10a2Uint = 26,
        R9g9b9e5Float = 27,
        R16Float = 28,
        R16Unorm = 29,
        R16Snorm = 30,
        R16Uint = 31,
        R16Sint = 32,
        R16g16Float = 33,
        R16g16Unorm = 34,
        R16g16Snorm = 35,
        R16g16Uint = 36,
        R16g16Sint = 37,
        R16g16b16a16Float = 38,
        R16g16b16a16Unorm = 39,
        R16g16b16a16Snorm = 40,
        R16g16b16a16Uint = 41,
        R16g16b16a16Sint = 42,
        R32Float = 43,
        R32Uint = 44,
        R32Sint = 45,
        R32g32Float = 46,
        R32g32Uint = 47,
        R32g32Sint = 48,
        R32g32b32Float = 49,
        R32g32b32Uint = 50,
        R32g32b32Sint = 51,
        R32g32b32a32Float = 52,
        R32g32b32a32Uint = 53,
        R32g32b32a32Sint = 54,
        Bc1Unorm = 55,
        Bc1Srgb = 56,
        Bc1aUnorm = 57,
        Bc1aSrgb = 58,
        Bc2Unorm = 59,
        Bc2Srgb = 60,
        Bc3Unorm = 61,
        Bc3Srgb = 62,
        Bc4Unorm = 63,
        Bc4Snorm = 64,
        Bc5Unorm = 65,
        Bc5Snorm = 66,
        Bc6uFloat = 67,
        Bc6sFloat = 68,
        Bc7Unorm = 69,
        Bc7Srgb = 70,
        Astc4x4Unorm = 71,
        Astc4x4Srgb = 72,
        Astc5x4Unorm = 73,
        Astc5x4Srgb = 74,
        Astc5x5Unorm = 75,
        Astc5x5Srgb = 76,
        Astc6x5Unorm = 77,
        Astc6x5Srgb = 78,
        Astc6x6Unorm = 79,
        Astc6x6Srgb = 80,
        Astc8x5Unorm = 81,
        Astc8x5Srgb = 82,
        Astc8x6Srgb = 83,
        Astc8x6Unorm = 84,
        Astc8x8Unorm = 85,
        Astc8x8Srgb = 86,
        Astc10x5Unorm = 87,
        Astc10x5Srgb = 88,
        Astc10x6Unorm = 89,
        Astc10x6Srgb = 90,
        Astc10x8Unorm = 91,
        Astc10x8Srgb = 92,
        Astc10x10Unorm = 93,
        Astc10x10Srgb = 94,
        Astc12x10Unorm = 95,
        Astc12x10Srgb = 96,
        Astc12x12Unorm = 97,
        Astc12x12Srgb = 98,
        D24UnormS8Uint = 99,
        D32FloatS8Uint = 100,
        D16Unorm = 101,
        D32Float = 102,
    }
}

impl Default for RenderFormat {
    fn default() -> Self {
        RenderFormat::Unknown
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderFormatInfo {
    pub block_width: u32,
    pub block_height: u32,
    pub block_bits: u32,
    pub red_bits: u32,
    pub green_bits: u32,
    pub blue_bits: u32,
    pub alpha_bits: u32,
    pub depth_bits: u32,
    pub stencil_bits: u32,
    pub padding_bits: u32,
    pub exponent_bits: u32,
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u32)]
    pub enum RenderNumericFormat {
        Unknown = 0,

        /// 16 or 32-bit floating-point
        Float = 1,

        /// Unsigned normalized integer in sRGB gamma. same as UNORM but data will be converted from sRGB to linear for the calculations
        Srgb = 2,

        /// Unsigned normalized integer. Component values in the range [0.0, 1.0] are stored as [0, MAX_INT], where MAX_INT is the greatest positive integer that can be stored, given the bit depth of the component.
        Unorm = 3,

        /// Two's complement signed normalized integer. Component values in the range [-1.0, 1.0] are stored as [MIN_INT, MAX_INT], where MIN_INT is the greatest negative integer and MAX_INT is the greatest positive integer that can be stored, given the bit depth of the component.
        Snorm = 4,

        /// Unsigned integer. Component values are stored in the range [0, MAX_INT], where MAX_INT is the greatest positive integer that can be stored, given the bit depth of the component.
        Uint = 5,

        /// Two's complement signed integer. Component values are stored in the range [MIN_INT, MAX_INT], where MIN_INT is the greatest negative integer and MAX_INT is the greatest positive integer that can be stored, given the bit depth of the component.
        Sint = 6,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u32)]
    pub enum RenderChannelFormat {
        Unknown = 0,
        R4G4 = 1,
        R4G4B4A4 = 2,
        R5G6B5 = 3,
        R5G5B5A1 = 4,
        R8 = 5,
        R8G8 = 6,
        R8G8B8 = 7, // Typically not a hardware format
        R8G8B8A8 = 8,
        B8G8R8A8 = 9,
        R10G11B11 = 10,
        R11G11B10 = 11,
        R10G10B10A2 = 12,
        R9G9B9E5 = 13,
        R16 = 14,
        R16G16 = 15,
        R16G16B16A16 = 16,
        R32 = 17,
        R32G32 = 18,
        R32G32B32 = 19,
        R32G32B32A32 = 20,
        BC1 = 21,
        BC1A = 22,
        BC2 = 23,
        BC3 = 24,
        BC4 = 25,
        BC5 = 26,
        BC6U = 27,
        BC6S = 28,
        BC7 = 29,
        Astc4x4 = 30,
        Astc5x4 = 31,
        Astc5x5 = 32,
        Astc6x5 = 33,
        Astc6x6 = 34,
        Astc8x5 = 35,
        Astc8x6 = 36,
        Astc8x8 = 37,
        Astc10x5 = 38,
        Astc10x6 = 39,
        Astc10x8 = 40,
        Astc10x10 = 41,
        Astc12x10 = 42,
        Astc12x12 = 43,
        D24S8 = 44,
        D32S8 = 45,
        D16 = 46,
        D24 = 47,
        D32 = 48,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderTextureType {
    Tex1d = 0,
    Tex1dArray = 1,
    Tex2d = 2,
    Tex2dArray = 3,
    Tex3d = 4,
    Cube = 5,
    CubeArray = 6,
}

impl Default for RenderTextureType {
    fn default() -> Self {
        RenderTextureType::Tex1d
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderViewDimension {
    Unknown = 0,
    Buffer = 1,
    Tex1d = 2,
    Tex1dArray = 3,
    Tex2d = 4,
    Tex2dArray = 5,
    Tex2dMs = 6,
    Tex2dMsArray = 7,
    Tex3d = 8,
    Cube = 9,
    CubeArray = 10,
    TopLevelAccelerationStructure = 11,
}

impl Default for RenderViewDimension {
    fn default() -> Self {
        RenderViewDimension::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
pub enum RenderPrimitiveType {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    TriangleList = 3,
    TriangleStrip = 4,
    QuadList = 5,
    RectList = 6,
    TrianglePatch = 7,
}

impl Default for RenderPrimitiveType {
    fn default() -> Self {
        RenderPrimitiveType::PointList
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderBox {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
    pub h: i32,
    pub d: i32,
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u16)]
    pub enum RenderResourceType {
        SwapChain = 0,
        Buffer = 1,
        Texture = 2,
        SamplerState = 3,
        Shader = 4,
        ShaderViews = 5,
        GraphicsPipelineState = 6,
        ComputePipelineState = 7,
        RayTracingGeometry = 8,
        RayTracingProgram = 9,
        RayTracingAcceleration = 10,
        RayTracingPipelineState = 11,
        RayTracingShaderTable = 12,
        DrawBindingSet = 13,
        FrameBindingSet = 14,
        RenderPass = 15,
        CommandList = 16,
        Fence = 17,
        TimingHeap = 18,
    }
}

impl Default for RenderResourceType {
    fn default() -> Self {
        RenderResourceType::SwapChain
    }
}

impl RenderResourceType {
    pub fn iter() -> Iter<'static, RenderResourceType> {
        static TYPES: [RenderResourceType; RENDERRESOURCETYPE_COUNT] = [
            RenderResourceType::SwapChain,
            RenderResourceType::Buffer,
            RenderResourceType::Texture,
            RenderResourceType::SamplerState,
            RenderResourceType::Shader,
            RenderResourceType::ShaderViews,
            RenderResourceType::GraphicsPipelineState,
            RenderResourceType::ComputePipelineState,
            RenderResourceType::RayTracingGeometry,
            RenderResourceType::RayTracingProgram,
            RenderResourceType::RayTracingAcceleration,
            RenderResourceType::RayTracingPipelineState,
            RenderResourceType::RayTracingShaderTable,
            RenderResourceType::DrawBindingSet,
            RenderResourceType::FrameBindingSet,
            RenderResourceType::RenderPass,
            RenderResourceType::CommandList,
            RenderResourceType::Fence,
            RenderResourceType::TimingHeap,
        ];
        TYPES.into_iter()
    }
}

bitflags! {
    pub struct RenderBindFlags: u16 {
        const NONE = 0x0000;
        const VERTEX_BUFFER = 0x0001;
        const INDEX_BUFFER = 0x0002;
        const CONSTANT_BUFFER = 0x0004;
        const INDIRECT_BUFFER = 0x0008;
        const SHADER_RESOURCE = 0x0010;
        const STREAM_OUTPUT = 0x0020;
        const RENDER_TARGET = 0x0040;
        const DEPTH_STENCIL = 0x0080;
        const UNORDERED_ACCESS = 0x0100;
        const CROSS_DEVICE = 0x0200;
        const ACCELERATION_STRUCTURE = 0x0400;
    }
}

impl Default for RenderBindFlags {
    fn default() -> Self {
        Self::NONE
    }
}

bitflags! {
    pub struct RenderResourceStates: u16 {
        const COMMON = 0;
        const VERTEX_AND_CONSTANT_BUFFER = 0x1;
        const INDEX_BUFFER = 0x2;
        const RENDER_TARGET = 0x4;
        const UNORDERED_ACCESS = 0x8;
        const DEPTH_WRITE = 0x10;
        const DEPTH_READ = 0x20;
        const NON_PIXEL_SHADER_RESOURCE = 0x40;
        const PIXEL_SHADER_RESOURCE = 0x80;
        const STREAM_OUT = 0x100;
        const INDIRECT_ARGUMENT = 0x200;
        const COPY_DEST = 0x400;
        const COPY_SOURCE = 0x800;
        const RESOLVE_DEST = 0x1000;
        const RESOLVE_SOURCE = 0x2000;
        const GENERIC_READ = (((((0x1 | 0x2) | 0x40) | 0x80) | 0x200) | 0x800);
        const PREDICATION = 0x200;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RenderDrawPacket {
    pub index_offset: u32,
    pub vertex_offset: i32,
    pub vertex_count: u32,
    pub first_instance: u32,
    pub instance_count: u32,
}

//#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Clone)]
pub struct RenderSwapChainDesc {
    pub width: u32,
    pub height: u32,
    pub format: RenderFormat,
    pub buffer_count: u32,
    pub window: RenderSwapChainWindow,
    //pub window: &'a winit::Window,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RenderSwapChainWindow {
    pub hinstance: *const std::os::raw::c_void,
    pub hwnd: *const std::os::raw::c_void,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RenderSwapChainWindow {
    pub ns_view: *const std::os::raw::c_void,
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RenderSwapChainWindow {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RenderShaderParameter {
    pub shader_resource_count: u32,
    pub unordered_access_count: u32,
    /// ex. "spaceN" in HLSL. Negative means auto-assign depending on parameter index (0, 1, 2...)
    pub register_space: i32,
}

impl RenderShaderParameter {
    #[inline(always)]
    fn new(shader_resource_count: u32, unordered_access_count: u32) -> Self {
        RenderShaderParameter {
            shader_resource_count,
            unordered_access_count,
            register_space: -1,
        }
    }

    #[inline(always)]
    fn new_with_space(
        shader_resource_count: u32,
        unordered_access_count: u32,
        register_space: i32,
    ) -> Self {
        RenderShaderParameter {
            shader_resource_count,
            unordered_access_count,
            register_space,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd)]
pub struct RenderShaderArgument {
    /// Allowed to be None if not used by shader
    pub constant_buffer: Option<RenderResourceHandle>,

    /// Allowed to be None if not used by shader
    pub shader_views: Option<RenderResourceHandle>,

    /// Must be a multiple of cbuffer alignment (256)
    pub constant_buffer_offset: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderShaderSignatureDesc {
    pub parameters: [RenderShaderParameter; MAX_SHADER_PARAMETERS],
    pub parameter_count: u32,

    // TODO: Could support many more bindings here but would want user to provide the pointer/memory as needed
    pub static_samplers: [RenderSamplerState; MAX_SAMPLER_BINDINGS],
    pub static_sampler_count: u32,
}

impl RenderShaderSignatureDesc {
    #[inline(always)]
    fn new(parameters: &[RenderShaderParameter], static_samplers: &[RenderSamplerState]) -> Self {
        // TODO: Improve
        assert!(parameters.len() <= MAX_SHADER_PARAMETERS);
        assert!(static_samplers.len() < MAX_SAMPLER_BINDINGS);

        let mut result = RenderShaderSignatureDesc {
            parameter_count: cmp::min(parameters.len(), MAX_SHADER_PARAMETERS) as u32,
            static_sampler_count: cmp::min(static_samplers.len(), MAX_SAMPLER_BINDINGS) as u32,
            ..Default::default()
        };

        for index in 0..result.parameter_count {
            result.parameters[index as usize] = parameters[index as usize];
        }

        for index in 0..result.static_sampler_count {
            result.static_samplers[index as usize] = static_samplers[index as usize];
        }

        result
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u8)]
    pub enum RenderShaderType {
        Vertex = 0,
        Geometry = 1,
        Hull = 2,
        Domain = 3,
        Pixel = 4,
        Compute = 5,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u8)]
    pub enum RenderCommandListType {
        Invalid = 0,
        Universal = 1,
        Compute = 2,
        Transfer = 3,
        Present = 4,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u8)]
    pub enum RayTracingShaderType {
        RayGen = 0,
        Miss = 1,
        IntersectionHit = 2,
        AnyHit = 3,
        ClosestHit = 4,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u8)]
    pub enum RayTracingProgramType {
        RayGen = 0,
        Miss = 1,
        Hit = 2,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u8)]
    pub enum RayTracingGeometryType {
        Triangle = 0,
        BoundingBox = 1,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u16)]
    pub enum RenderLoadOp {
        Discard = 0,
        Load = 1,
        Clear = 2,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, EnumCount)]
    #[repr(u16)]
    pub enum RenderStoreOp {
        Discard = 0,
        Store = 1,
    }
}

bitflags! {
    pub struct RenderDepthStencilViewFlags: u16 {
        const NONE = 0x0;
        const READ_ONLY_DEPTH = 0x1;
        const READ_ONLY_STENCIL = 0x2;
    }
}

impl Default for RenderDepthStencilViewFlags {
    fn default() -> Self {
        RenderDepthStencilViewFlags::NONE
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct RenderShaderDesc {
    pub shader_type: RenderShaderType,
    pub shader_data: Vec<u8>,
}

#[derive(Clone, Default, Debug)]
pub struct RenderBufferDesc {
    pub bind_flags: RenderBindFlags,
    pub size: usize,
}

#[derive(Clone, Default, Debug)]
pub struct RenderTextureDesc {
    pub texture_type: RenderTextureType,
    pub bind_flags: RenderBindFlags,
    pub format: RenderFormat,
    pub width: u32,
    pub height: u32,
    pub depth: u16,
    pub levels: u16,
    pub elements: u16,
    //pub samples: u16,
}

#[derive(Debug)]
pub struct RenderTextureSubResourceData<'a> {
    pub data: &'a Vec<u8>,
    pub row_pitch: u32,
    pub slice_pitch: u32,
}

#[derive(Clone, Default, Debug)]
pub struct RenderShaderViewsDesc {
    // No CBV here as it is passed in dynamically at bind time
    pub shader_resource_views: Vec<RenderBindingShaderResourceView>,
    pub unordered_access_views: Vec<RenderBindingUnorderedAccessView>,
}

// TODO: Should make this just use generic shader and program desc types
#[derive(Clone, Default, Debug)]
pub struct RayTracingShaderDesc {
    pub entry_point: String,
    pub shader_data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RayTracingProgramDesc {
    pub program_type: RayTracingProgramType,
    pub shaders: [RayTracingShaderDesc; MAX_RAY_TRACING_SHADER_TYPE],
    pub signature: RenderShaderSignatureDesc,
}

#[derive(Clone, Debug)]
pub struct RayTracingGeometryPart {
    pub index_count: u32,
    pub index_offset: u32,
}

#[derive(Clone, Debug)]
pub struct RayTracingGeometryDesc {
    pub geometry_type: RayTracingGeometryType,
    pub vertex_buffer: RenderBindingBuffer,
    pub index_buffer: RenderBindingBuffer,
    pub parts: Vec<RayTracingGeometryPart>,
}

#[derive(Clone, Debug)]
pub struct RayTracingTopAccelerationDesc {}

#[derive(Clone, Debug)]
pub struct RayTracingBottomAccelerationDesc {}

#[derive(Clone, Debug)]
pub struct RayTracingPipelineStateDesc {
    pub programs: Vec<RenderResourceHandle>,
}

#[derive(Clone, Debug)]
pub struct RayTracingShaderTableDesc {
    pub raygen_entry_count: u32,
    pub hit_entry_count: u32,
    pub miss_entry_count: u32,
}

#[derive(Clone, Debug)]
pub struct RenderDrawBindingSetDesc {
    pub vertex_buffers: [Option<RenderBindingBuffer>; MAX_VERTEX_STREAMS],
    pub index_buffer: Option<RenderBindingBuffer>,
}

#[derive(Clone, Default, Debug)]
pub struct RenderFrameBindingSetDesc {
    pub render_target_views: [Option<RenderBindingRenderTargetView>; MAX_RENDER_TARGET_COUNT],
    pub depth_stencil_view: Option<RenderBindingDepthStencilView>,
}

#[derive(Clone, Debug)]
pub struct RenderTargetInfo {
    pub load_op: RenderLoadOp,
    pub store_op: RenderStoreOp,
    pub clear_color: [f32; 4],
}

impl Default for RenderTargetInfo {
    fn default() -> Self {
        RenderTargetInfo {
            load_op: RenderLoadOp::Load,
            store_op: RenderStoreOp::Store,
            clear_color: [0f32, 0f32, 0f32, 0f32],
        }
    }
}

#[derive(Clone, Debug)]
pub struct DepthStencilTargetInfo {
    pub load_op: RenderLoadOp,
    pub store_op: RenderStoreOp,
    pub clear_depth: f32,
    pub clear_stencil: u8,
}

impl Default for DepthStencilTargetInfo {
    fn default() -> Self {
        DepthStencilTargetInfo {
            load_op: RenderLoadOp::Load,
            store_op: RenderStoreOp::Store,
            clear_depth: 1f32,
            clear_stencil: 0u8,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct RenderPassDesc {
    pub frame_binding: RenderResourceHandle,
    pub render_target_info: [RenderTargetInfo; MAX_RENDER_TARGET_COUNT],
    pub depth_stencil_target_info: DepthStencilTargetInfo,
}

#[derive(Clone, Debug)]
pub struct RenderFenceDesc {
    pub cross_device: bool,
}

#[derive(Clone, Debug)]
pub struct RenderTimingHeapDesc {
    /// Number of begin/end pairs
    pub region_count: usize,
}

#[derive(Clone, Debug)]
pub struct RenderDrawArguments {
    pub vertex_count_per_instance: u32,
    pub instance_count: u32,
    pub start_vertex_location: u32,
    pub start_instance_location: u32,
}

#[derive(Clone, Debug)]
pub struct RenderDrawIndexedArguments {
    pub index_count_per_instance: u32,
    pub instance_count: u32,
    pub start_vertex_location: u32,
    pub base_vertex_location: i32,
    pub start_instance_location: u32,
}

#[derive(Clone, Debug)]
pub struct RenderDispatchArguments {
    pub thread_group_count_x: u32,
    pub thread_group_count_y: u32,
    pub thread_group_count_z: u32,
}

#[derive(Clone, Debug)]
pub struct RenderUploadHeapDesc {
    pub heap_size: usize,
}

#[inline(always)]
pub fn get_resource_states(bind_flags: RenderBindFlags) -> RenderResourceStates {
    let mut result = RenderResourceStates::COMMON;

    if bind_flags.contains(RenderBindFlags::VERTEX_BUFFER)
        || bind_flags.contains(RenderBindFlags::CONSTANT_BUFFER)
    {
        result |= RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER;
    }

    if bind_flags.contains(RenderBindFlags::INDEX_BUFFER) {
        result |= RenderResourceStates::INDEX_BUFFER;
    }

    if bind_flags.contains(RenderBindFlags::INDIRECT_BUFFER) {
        result |= RenderResourceStates::INDIRECT_ARGUMENT;
    }

    if bind_flags.contains(RenderBindFlags::SHADER_RESOURCE) {
        result |= RenderResourceStates::NON_PIXEL_SHADER_RESOURCE
            | RenderResourceStates::PIXEL_SHADER_RESOURCE;
    }

    if bind_flags.contains(RenderBindFlags::STREAM_OUTPUT) {
        result |= RenderResourceStates::STREAM_OUT;
    }

    if bind_flags.contains(RenderBindFlags::RENDER_TARGET) {
        result |= RenderResourceStates::RENDER_TARGET;
    }

    if bind_flags.contains(RenderBindFlags::DEPTH_STENCIL) {
        result |= RenderResourceStates::DEPTH_WRITE | RenderResourceStates::DEPTH_READ;
    }

    if bind_flags.contains(RenderBindFlags::UNORDERED_ACCESS) {
        result |= RenderResourceStates::UNORDERED_ACCESS;
    }

    return result;
}

#[inline(always)]
pub fn get_default_resource_states(bind_flags: RenderBindFlags) -> RenderResourceStates {
    let mut result = RenderResourceStates::COMMON;

    if bind_flags.contains(RenderBindFlags::VERTEX_BUFFER)
        || bind_flags.contains(RenderBindFlags::CONSTANT_BUFFER)
    {
        result |= RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER;
    }

    if bind_flags.contains(RenderBindFlags::INDEX_BUFFER) {
        result |= RenderResourceStates::INDEX_BUFFER;
    }

    if bind_flags.contains(RenderBindFlags::INDIRECT_BUFFER) {
        result |= RenderResourceStates::INDIRECT_ARGUMENT;
    }

    if bind_flags.contains(RenderBindFlags::SHADER_RESOURCE) {
        result |= RenderResourceStates::NON_PIXEL_SHADER_RESOURCE
            | RenderResourceStates::PIXEL_SHADER_RESOURCE;
    }

    if bind_flags.contains(RenderBindFlags::STREAM_OUTPUT) {
        result |= RenderResourceStates::STREAM_OUT;
    }

    if bind_flags.contains(RenderBindFlags::RENDER_TARGET) {
        result |= RenderResourceStates::RENDER_TARGET;
    }

    if bind_flags.contains(RenderBindFlags::DEPTH_STENCIL) {
        result |= RenderResourceStates::DEPTH_WRITE; // TODO: Only difference with regular method is lack of DEPTH_READ here
    }

    if bind_flags.contains(RenderBindFlags::UNORDERED_ACCESS) {
        result |= RenderResourceStates::UNORDERED_ACCESS;
    }

    result
}

#[derive(Default)]
pub struct RenderUploadHeapRange {
    pub upload_heap: RenderResourceHandle,
    pub offset: usize,
    pub size: usize,
}

pub struct RenderSharedResource {
    // TODO: pub handle: Handle,
}

pub struct RenderTextureLayoutInfo {
    pub pitch: u32,
    pub slice_pitch: u32,
}

#[inline(always)]
pub fn calc_texture_sub_resource_index(mip_index: u32, slice_index: u32, mip_count: u32) -> u32 {
    mip_index + mip_count * slice_index
}

#[inline(always)]
pub fn get_texture_sub_resource_mip_index(sub_resource_index: u32, mip_count: u32) -> u32 {
    sub_resource_index % mip_count
}

#[inline(always)]
pub fn get_texture_sub_resource_slice_index(sub_resource_index: u32, mip_count: u32) -> u32 {
    sub_resource_index / mip_count
}

#[inline(always)]
pub fn get_texture_max_mip_count(width: u32, height: u32, depth: u32) -> u32 {
    use std::cmp::max;
    let max_dim = max(max(width, height), depth) as f32;
    1 + max_dim.log2() as u32
}

#[inline(always)]
pub fn get_texture_sub_resource_count(desc: &RenderTextureDesc) -> u32 {
    let mut sub_resource_count = desc.levels * desc.elements;
    if desc.texture_type == RenderTextureType::Cube
        || desc.texture_type == RenderTextureType::CubeArray
    {
        sub_resource_count *= 6;
    }
    sub_resource_count as u32
}
