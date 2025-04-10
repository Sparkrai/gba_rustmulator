use std::path::Path;

use glium::glutin;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::Display;
use imgui::{Context, FontConfig, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

mod clipboard;

pub struct System {
	pub event_loop: EventLoop<()>,
	pub display: glium::Display,
	pub imgui: Context,
	pub platform: WinitPlatform,
	pub renderer: Renderer,
	pub font_size: f32,
}

pub fn init(title: &str) -> System {
	let title = match Path::new(&title).file_name() {
		Some(file_name) => file_name.to_str().unwrap(),
		None => title,
	};
	let event_loop = EventLoop::new();
	let context = glutin::ContextBuilder::new().with_vsync(false);
	let builder = WindowBuilder::new().with_title(title.to_owned()).with_maximized(true);
	let display = Display::new(builder, context, &event_loop).expect("Failed to initialize display");

	let mut imgui = Context::create();
	imgui.set_ini_filename(None);

	if let Some(backend) = clipboard::init() {
		imgui.set_clipboard_backend(Box::new(backend));
	} else {
		eprintln!("Failed to initialize clipboard");
	}

	let mut platform = WinitPlatform::init(&mut imgui);
	{
		let gl_window = display.gl_window();
		let window = gl_window.window();
		platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
	}

	let hidpi_factor = platform.hidpi_factor();
	let font_size = (13.0 * hidpi_factor) as f32;
	imgui.fonts().add_font(&[FontSource::DefaultFontData {
		config: Some(FontConfig {
			size_pixels: font_size,
			..FontConfig::default()
		}),
	}]);

	imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

	let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

	System {
		event_loop,
		display,
		imgui,
		platform,
		renderer,
		font_size,
	}
}
