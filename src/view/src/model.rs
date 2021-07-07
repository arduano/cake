use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use backend::CakeBackendModel;
use util::fps::Fps;

pub struct CakeViewModel {
    pub fps: Fps,
    pub paused: bool,
}

impl CakeViewModel {
    pub fn new() -> Self {
        CakeViewModel {
            fps: Fps::new(),
            paused: true,
        }
    }
}

pub struct CakeModel {
    pub backend: Arc<Mutex<CakeBackendModel>>,
    pub view: CakeViewModel,
}

impl CakeModel {
    pub fn new() -> Self {
        CakeModel {
            backend: Arc::new(Mutex::new(CakeBackendModel {})),
            view: CakeViewModel::new(),
        }
    }
}
