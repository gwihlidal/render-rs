use ash;
use render_core::state::{RenderBindingShaderResourceView, RenderBindingUnorderedAccessView};

#[derive(Debug, Default, Copy, Clone)]
pub struct ShaderResourceViewBinding {
    pub desc: RenderBindingShaderResourceView,
    pub image_view: ash::vk::ImageView,
    pub buffer_view: ash::vk::BufferView,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct UnorderedAccessViewBinding {
    pub desc: RenderBindingUnorderedAccessView,
    pub image_view: ash::vk::ImageView,
    pub buffer_view: ash::vk::BufferView,
}
