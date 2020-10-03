use core::ffi::c_void;
use gio::prelude::*;
use glium::backend::Context;
use glium::Surface;
use glium::SwapBuffersError;
use gtk::prelude::*;
use std::env::args;

struct GLAreaBackend {
    glarea: gtk::GLArea,
}

unsafe impl glium::backend::Backend for GLAreaBackend {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        // GTK swaps the buffers after each "render" signal itself
        Ok(())
    }
    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        gl_loader::get_proc_address(symbol) as *const _
    }
    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let allocation = self.glarea.get_allocation();
        (allocation.width as u32, allocation.height as u32)
    }
    fn is_current(&self) -> bool {
        // GTK makes it current itself on each "render" signal
        true
    }
    unsafe fn make_current(&self) {
        self.glarea.make_current();
    }
}

impl GLAreaBackend {
    fn new(glarea: gtk::GLArea) -> Self {
        Self { glarea }
    }
}

fn main() {
    let application =
        gtk::Application::new(None, Default::default()).expect("Initialization failed");

    application.connect_activate(|app| {
        let window = gtk::ApplicationWindowBuilder::new()
            .application(app)
            .title("gtk-glium")
            .window_position(gtk::WindowPosition::Center)
            .default_width(600)
            .default_height(400)
            .build();

        let glarea = gtk::GLArea::new();
        window.add(&glarea);
        window.show_all();

        // load gl
        gl_loader::init_gl();

        // create glium context
        let context = unsafe {
            Context::new(
                GLAreaBackend::new(glarea.clone()),
                true,
                glium::debug::DebugCallbackBehavior::DebugMessageOnError,
            )
            .unwrap()
        };

        let counter = std::rc::Rc::new(std::cell::RefCell::new(0u32));

        glarea.connect_render(move |_glarea, _glcontext| {
            let mut frame =
                glium::Frame::new(context.clone(), context.get_framebuffer_dimensions());
            // this is where you can do your glium rendering

            let c = *counter.borrow() as f32 / 100.;
            let r = c.sin() / 2. + 0.5;
            let g = (c * 1.25).sin() / 2. + 0.5;
            let b = (c * 1.5).sin() / 2. + 0.5;
            frame.clear_color(r, g, b, 1.0);

            frame.finish().unwrap();
            *counter.borrow_mut() += 1;
            Inhibit(true)
        });

        // This makes the GLArea redraw 60 times per second
        // You can remove this if you want to redraw only when focused/resized
        const FPS: u32 = 60;
        glib::source::timeout_add_local(1_000 / FPS, move || {
            glarea.queue_draw();
            glib::source::Continue(true)
        });
    });

    application.run(&args().collect::<Vec<_>>());
}
