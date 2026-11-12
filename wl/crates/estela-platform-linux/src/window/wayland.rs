use std::{cell::RefCell, ffi::c_void, mem::ManuallyDrop};

use wayland_client::{
    backend::{Backend, ObjectId},
    protocol::{wl_compositor, wl_region, wl_registry, wl_surface},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};

use estela_capture::{window::WindowInput, Error, Result};

thread_local! {
    // Explicit release keeps foreign displays alive until owned proxies are gone.
    static CONTEXT: RefCell<Option<ManuallyDrop<Context>>> = const { RefCell::new(None) };
}

struct Context {
    display: *mut c_void,
    connection: Connection,
    compositor: wl_compositor::WlCompositor,
    queue: EventQueue<Globals>,
    _registry: wl_registry::WlRegistry,
}

#[derive(Default)]
struct Globals {
    compositor: Option<wl_compositor::WlCompositor>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for Globals {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_compositor" {
                state.compositor = Some(registry.bind(name, version.min(4), queue, ()));
            }
        }
    }
}

wayland_client::delegate_noop!(Globals: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(Globals: ignore wl_region::WlRegion);

pub(super) fn configure(
    surface: *mut c_void,
    display: *mut c_void,
    input: &WindowInput,
) -> Result<()> {
    CONTEXT.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage.is_none() {
            *storage = Some(ManuallyDrop::new(Context::new(display)?));
        }
        let context = storage
            .as_ref()
            .ok_or_else(|| backend("Missing Wayland context"))?;
        if context.display != display {
            return Err(backend("Wayland display changed before cleanup"));
        }
        // SAFETY: Iced borrows its live surface on the event loop thread.
        let id = unsafe { ObjectId::from_ptr(wl_surface::WlSurface::interface(), surface.cast()) }
            .map_err(backend)?;
        let surface = wl_surface::WlSurface::from_id(&context.connection, id).map_err(backend)?;
        match input {
            WindowInput::Entire => surface.set_input_region(None),
            WindowInput::Areas(areas) => {
                let region = context
                    .compositor
                    .create_region(&context.queue.handle(), ());
                for rectangle in areas.iter().flat_map(|area| area.rectangles(1.0)) {
                    if rectangle.width > 0 && rectangle.height > 0 {
                        region.add(
                            rectangle.x,
                            rectangle.y,
                            rectangle.width as i32,
                            rectangle.height as i32,
                        );
                    }
                }
                surface.set_input_region(Some(&region));
                region.destroy();
            }
        }
        surface.commit();
        context.connection.flush().map_err(backend)
    })
}

impl Context {
    fn new(display: *mut c_void) -> Result<Self> {
        // SAFETY: The recorder retains Iced's display until explicit release.
        let native_backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let connection = Connection::from_backend(native_backend);
        let mut queue = connection.new_event_queue();
        let registry = connection.display().get_registry(&queue.handle(), ());
        let mut globals = Globals::default();
        queue.roundtrip(&mut globals).map_err(backend)?;
        let compositor = globals
            .compositor
            .ok_or_else(|| backend("Wayland compositor unavailable"))?;
        Ok(Self {
            display,
            connection,
            compositor,
            queue,
            _registry: registry,
        })
    }
}

pub(super) fn release(display: *mut c_void) {
    CONTEXT.with(|storage| {
        let mut storage = storage.borrow_mut();
        if storage
            .as_ref()
            .is_some_and(|context| context.display == display)
        {
            if let Some(context) = storage.take() {
                drop(ManuallyDrop::into_inner(context));
            }
        }
    });
}

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}
