use ash::{version::DeviceV1_0, vk};
use legion::system;

use crate::{
    components::{Mesh, TransformMatrix},
    resources::VulkanContext,
    resources::{render_context::create_push_constant, RenderContext},
};

#[system(for_each)]
pub(crate) fn rendering(
    mesh: &Mesh,
    transform_matrix: &TransformMatrix,
    #[resource] vulkan_context: &VulkanContext,
    #[resource] swapchain_image_index: &usize,
    #[resource] render_context: &RenderContext,
) -> () {
    let device = &vulkan_context.device;
    let command_buffer = render_context.frames[*swapchain_image_index].command_buffer;

    unsafe {
        // Bind mesh descriptor sets
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            render_context.pipeline_layout,
            1,
            &mesh.descriptor_sets,
            &[],
        );

        // Bind vertex and index buffers
        device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer.handle], &[0]);
        device.cmd_bind_index_buffer(
            command_buffer,
            mesh.index_buffer.handle,
            0,
            vk::IndexType::UINT32,
        );

        // Push constants
        let model_matrix = create_push_constant(&transform_matrix.0);
        device.cmd_push_constants(
            command_buffer,
            render_context.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            model_matrix,
        );
        device.cmd_draw_indexed(command_buffer, mesh.num_indices, 1, 0, 0, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legion::{Resources, Schedule, World};

    use crate::resources::{RenderContext, XrContext};

    #[test]
    pub fn test_rendering_system() {
        let mut world = World::default();
        let (xr_context, vulkan_context) = XrContext::new().unwrap();
        let render_context = RenderContext::new(&vulkan_context, &xr_context).unwrap();

        let mut schedule = Schedule::builder().add_system(rendering_system()).build();
        let mut resources = Resources::default();
        resources.insert(vulkan_context);
        resources.insert(render_context);
        resources.insert(0 as usize);
        schedule.execute(&mut world, &mut resources);

        let mut frame_index = resources.get_mut::<usize>().unwrap();
        (*frame_index) = 1;
        drop(frame_index);
        schedule.execute(&mut world, &mut resources);
    }
}
