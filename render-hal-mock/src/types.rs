#![allow(dead_code)]

use crate::device::RenderDeviceMock;
use render_core::error::{Error, Result};
use render_core::resources::RenderResourceBase;
use render_core::types::*;
use std::{borrow::Cow, fmt, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderShaderMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderShaderMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Shader
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct RenderSwapChainMock {
    pub name: Cow<'static, str>,
    pub textures: Vec<RenderTextureMock>,
    pub back_buffer_index: u32,
}

impl RenderResourceBase for RenderSwapChainMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::SwapChain
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderBufferMock {
    pub name: Cow<'static, str>,
    pub desc: RenderBufferDesc,
}

impl RenderResourceBase for RenderBufferMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Buffer
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderTextureMock {
    pub name: Cow<'static, str>,
    pub desc: RenderTextureDesc,
}

impl RenderResourceBase for RenderTextureMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Texture
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFenceMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderFenceMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Fence
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderSamplerStateMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderSamplerStateMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::SamplerState
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderGraphicsPipelineStateMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderGraphicsPipelineStateMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::GraphicsPipelineState
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderComputePipelineStateMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderComputePipelineStateMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::ComputePipelineState
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderShaderViewsMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderShaderViewsMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::ShaderViews
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderDrawBindingSetMock {
    pub name: Cow<'static, str>,
    //RenderDrawBindingSetDesc desc;
}

impl RenderResourceBase for RenderDrawBindingSetMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::DrawBindingSet
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderFrameBindingSetMock {
    pub name: Cow<'static, str>,
    pub desc: RenderFrameBindingSetDesc,
}

impl RenderResourceBase for RenderFrameBindingSetMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::FrameBindingSet
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderPassMock {
    pub name: Cow<'static, str>,
    pub desc: RenderPassDesc,
}

impl RenderResourceBase for RenderPassMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RenderPass
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingPipelineStateMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingPipelineStateMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingPipelineState
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingProgramMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingProgramMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingProgram
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingGeometryMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingGeometryMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingGeometry
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingAccelerationMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingAccelerationMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingAcceleration
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingShaderTableMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingShaderTableMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingShaderTable
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderTimingHeapMock {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderTimingHeapMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::TimingHeap
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderCommandListMock {
    pub name: Cow<'static, str>,
    //pub device: Arc<dyn RenderDeviceMock>,
    pub list_type: RenderCommandListType,
}

impl RenderCommandListMock {
    fn new(
        //device: Arc<dyn RenderDeviceMock>,
        list_type: RenderCommandListType,
        debug_name: Cow<'static, str>,
    ) -> Self {
        RenderCommandListMock {
            //device,
            list_type,
            name: debug_name,
        }
    }

    fn get(&mut self) -> Result<()> {
        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn submit(&mut self) -> Result<()> {
        Ok(())
    }

    //#[always_inline]
    fn is_open(&self) -> bool {
        false
    }

    //#[always_inline]
    fn list_type(&self) -> RenderCommandListType {
        self.list_type
    }
}

impl RenderResourceBase for RenderCommandListMock {
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::CommandList
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
