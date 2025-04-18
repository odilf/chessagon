use bytemuck::Zeroable as _;
use eframe::egui_wgpu::wgpu::{self, util::DeviceExt};
use eframe::egui_wgpu::{self, RenderState};
use std::num::NonZeroU64;

use super::Uniforms;

/// Prepares the gpu in the correct state.
///
/// It loads the shader and sets the binding groups for the uniforms.
pub fn prepare(wgpu_render_state: &RenderState) {
    let device = &wgpu_render_state.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("board"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./board.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("board"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(464),
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("board"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("board"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: None,
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: None,
            targets: &[Some(wgpu_render_state.target_format.into())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("board"),
        contents: bytemuck::bytes_of(&Uniforms::zeroed()), // 16 bytes aligned!
        // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
        // (this *happens* to workaround this bug )
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("board"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // Because the graphics pipeline must have the same lifetime as the egui render pass,
    // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
    // `paint_callback_resources` type map, which is stored alongside the render pass.
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(BoardRenderResources {
            pipeline,
            bind_group,
            uniform_buffer,
        });
}

pub struct BoardRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
}

impl BoardRenderResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, uniforms: &Uniforms) {
        let bytes = bytemuck::bytes_of(uniforms);
        queue.write_buffer(&self.uniform_buffer, 0, bytes);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        render_pass.draw(1..4, 0..1);
    }
}

// Callbacks in egui_wgpu have 3 stages:
// * prepare (per callback impl)
// * finish_prepare (once)
// * paint (per callback impl)
//
// The prepare callback is called every frame before paint and is given access to the wgpu
// Device and Queue, which can be used, for instance, to update buffers and uniforms before
// rendering.
// If [`egui_wgpu::Renderer`] has [`egui_wgpu::FinishPrepareCallback`] registered,
// it will be called after all `prepare` callbacks have been called.
// You can use this to update any shared resources that need to be updated once per frame
// after all callbacks have been processed.
//
// On both prepare methods you can use the main `CommandEncoder` that is passed-in,
// return an arbitrary number of user-defined `CommandBuffer`s, or both.
// The main command buffer, as well as all user-defined ones, will be submitted together
// to the GPU in a single call.
//
// The paint callback is called after finish prepare and is given access to egui's main render pass,
// which can be used to issue draw commands.
pub struct CustomBoardCallback {
    pub uniforms: Uniforms,
}

impl egui_wgpu::CallbackTrait for CustomBoardCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &BoardRenderResources = resources.get().unwrap();
        resources.prepare(device, queue, &self.uniforms);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &BoardRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}
