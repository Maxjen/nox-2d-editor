use std::time::Duration;
use imgui_winit_support::WinitPlatform;
use legion::*;
use winit::window::Window;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use crate::{
    wgpu_state::WgpuState,
    app_state::AppState,
    application::DeltaTime,
    events::Events,
    command::Command,
};

pub struct UiState {
    pub imgui: Context,
    pub platform: WinitPlatform,
    renderer: Renderer,
    last_cursor: Option<MouseCursor>,
    current_collection: usize,
}

impl UiState {
    pub fn new(window: &Window, wgpu_state: &WgpuState) -> Self {
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: wgpu_state.sc_desc.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, &wgpu_state.device, &wgpu_state.queue, renderer_config);

        Self {
            imgui,
            platform,
            renderer,
            last_cursor: None,
            current_collection: 0,
        }
    }

    pub fn render_ui(&mut self, window: &Window, resources: &Resources) {
        let delta_time = resources.get::<DeltaTime>().unwrap();
        let wgpu_state = resources.get::<WgpuState>().unwrap();
        let app_state = resources.get::<AppState>().unwrap();
        let mut commands = resources.get_mut::<Events<Command>>().unwrap();

        if let None = wgpu_state.current_frame { return; }
        let frame = &wgpu_state.current_frame.as_ref().unwrap().output;

        self.imgui.io_mut().update_delta_time(Duration::from_secs_f32(delta_time.0));

        self.platform
            .prepare_frame(self.imgui.io_mut(), &window)
            .expect("Failed to prepare frame");
        let ui = self.imgui.frame();

        {
            let mut current_collection = self.current_collection;

            let size = window.inner_size().to_logical::<f32>(window.scale_factor());
            let panel = imgui::Window::new(im_str!("Outliner"));
            panel
                .position([0.0, 0.0], Condition::FirstUseEver)
                .size([300.0, size.height], Condition::FirstUseEver)
                .build(&ui, || {
                    if CollapsingHeader::new(im_str!("Files"))
                        .default_open(true)
                        .build(&ui) {
                        let collection_indices = &app_state.data_accessor.collection_indices;
                        let mut collection_names: Vec<String> = vec![String::new(); collection_indices.len()];
                        for (name, index) in collection_indices {
                            let mut name = name.clone();
                            name.push_str("\0");
                            collection_names[*index] = name;
                        }
                        imgui::ListBox::new(im_str!(""))
                            .build_simple(&ui, &mut current_collection, collection_names.as_slice(), &|item: &String| {
                                let result = unsafe { ImStr::from_utf8_with_nul_unchecked(item.as_bytes()).into() };
                                //println!("{:?}", result);
                                return result;
                            });
                        /*ui.text(im_str!("Hello World!"));
                        ui.separator();
                        let mouse_pos = ui.io().mouse_pos;
                        ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos[0], mouse_pos[1]));
                        imgui::TreeNode::new(im_str!("Test"))
                            .default_open(true)
                            .build(&ui, || {
                                imgui::TreeNode::new(im_str!("Test2")).default_open(false).build(&ui, || {});
                            });*/
                    }
                }
            );

            if self.current_collection != current_collection {
                self.current_collection = current_collection;
                commands.send(Command::SetCurrentScene(current_collection));
            }
        }

        let mut encoder = wgpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UI Render Encoder"),
            });
        
        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, window);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                    store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(ui.render(), &wgpu_state.queue, &wgpu_state.device, &mut render_pass)
                .expect("Rendering failed");
        }
        wgpu_state.queue.submit(Some(encoder.finish()));
    }
}

