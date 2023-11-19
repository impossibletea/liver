use std::{
    rc::Rc,
    sync::Arc,
    cell::RefCell,
    os::raw::c_void,
};
use glium::{
    Frame,
    SwapBuffersError as SwapErr,
    debug::DebugCallbackBehavior,
    backend::{Backend, Context, Facade},
    glutin::{
        RawContext,
        ContextBuilder,
        PossiblyCurrent,
        ContextError as ContErr,
        platform::unix::{
            RawContextExt,
            x11::XConnection,
        },
    },
};
use takeable_option::Takeable;

// __  ______                           _               _
// \ \/ / ___|  ___  ___ _   _ _ __ ___| |    ___   ___| | __
//  \  /\___ \ / _ \/ __| | | | '__/ _ \ |   / _ \ / __| |/ /
//  /  \ ___) |  __/ (__| |_| | | |  __/ |__| (_) | (__|   <
// /_/\_\____/ \___|\___|\__,_|_|  \___|_____\___/ \___|_|\_\

pub struct XSecureLock (Rc<Context>);

impl XSecureLock {

    //  _ _   _ __   _____      __
    // (_|_) | '_ \ / _ \ \ /\ / /
    //  _ _  | | | |  __/\ V  V /
    // (_|_) |_| |_|\___| \_/\_/

    pub fn new(xwin: u64) -> Result<Self, String> {
        let backend = XSecureLockBack::new(xwin)?;
        let context = unsafe {
            Context::new(backend,
                         false,
                         DebugCallbackBehavior::Ignore)
            .map_err(|e| format!("Fucked up context: {e}"))
        }?;

        Ok(Self(context))
    }

    //            _
    //  _ _    __| |_ __ __ ___      __
    // (_|_)  / _` | '__/ _` \ \ /\ / /
    //  _ _  | (_| | | | (_| |\ V  V /
    // (_|_)  \__,_|_|  \__,_| \_/\_/

    pub fn draw(&self) -> Frame {
        Frame::new(self.0.clone(),
                   self.0.get_framebuffer_dimensions())
    }
}

impl Facade for XSecureLock {
    fn get_context(&self) -> &Rc<Context> {&self.0}
}

#[derive(Clone)]
struct XSecureLockBack {
    context: Rc<RefCell<Takeable<RawContext<PossiblyCurrent>>>>,
    xconn:   Arc<XConnection>,
    xwin:    u64,
}

impl XSecureLockBack {
    fn new(xwin: u64) -> Result<Self, String> {
        let xconn =
            XConnection::new(None)
            .map_err(|e| format!("Failed to connect to X: {e}"))
            .map(|x| Arc::new(x))?;
        let context = unsafe {
            ContextBuilder::new()
            .build_raw_x11_context(xconn.clone(),
                                   xwin)
            .map_err(|e| format!("Failed to create context: {e}"))
            .and_then(|c| c.make_current()
                          .map_err(|e| format!("Failed to current: {}", e.1)))
        }?;

        Ok(Self {
            context: Rc::new(RefCell::new(Takeable::new(context))),
            xconn,
            xwin,
        })
    }
}

unsafe impl Backend for XSecureLockBack {
   fn swap_buffers(&self) -> Result<(), SwapErr> {
       match self.context.borrow().swap_buffers() {
           Ok(())                    => Ok(()),
           Err(ContErr::ContextLost) => Err(SwapErr::ContextLost),
           Err(e)                    => panic!("wtf: {e}")
       }
   }

   unsafe fn get_proc_address(&self,
                              symbol: &str) -> *const c_void {
       self.context.borrow().get_proc_address(symbol)
   }

   fn get_framebuffer_dimensions(&self) -> (u32, u32) {
       let geometry = self.xconn.get_geometry(self.xwin);

       match geometry {
           Ok(g)  => (g.width, g.height),
           Err(_) => (0, 0)
       }
   }

   fn is_current(&self) -> bool {self.context.borrow().is_current()}

   unsafe fn make_current(&self) {
       let mut gl_window_takeable = self.context.borrow_mut();
       let gl_window = Takeable::take(&mut gl_window_takeable);
       let new_gl_window =
           gl_window.make_current()
           .unwrap_or_else(|(c, _)| c);
       Takeable::insert(&mut gl_window_takeable, new_gl_window);
   }
}
