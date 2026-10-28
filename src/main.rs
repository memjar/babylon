mod renderer;
mod hud;
mod huds;
mod protocol;
mod feed;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};
use winit::event::ElementState;
use winit::keyboard::{Key, NamedKey};
use renderer::Renderer;
use hud::{TacticalHud, HudMode};
use feed::TacticalFeed;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    hud: TacticalHud,
    feed: Option<TacticalFeed>,
}

impl App {
    fn new() -> Self {
        let feed_url = std::env::var("C2_FEED").unwrap_or_else(|_| "ws://127.0.0.1:9100/c2".into());
        let feed = TacticalFeed::connect(&feed_url);
        Self {
            window: None,
            renderer: None,
            hud: TacticalHud::new(),
            feed: Some(feed),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("HOLODECK — Tactical HUD")
            .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 800u32));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key.as_ref() {
                    Key::Character("1") => self.hud.mode = HudMode::Radar,
                    Key::Character("2") => self.hud.mode = HudMode::F35Hmds,
                    Key::Character("3") => self.hud.mode = HudMode::Sonar,
                    Key::Character("4") => self.hud.mode = HudMode::Apache,
                    Key::Character("5") => self.hud.mode = HudMode::Aegis,
                    Key::Character("6") => self.hud.mode = HudMode::Reaper,
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(r), Some(w)) = (self.renderer.as_mut(), self.window.as_ref()) {
                    if let Some(feed) = &self.feed {
                        self.hud.ingest_feed(&feed.snapshot());
                    }
                    self.hud.tick();
                    r.render(&self.hud);
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("run app");
}
