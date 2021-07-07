use std::sync::{Arc, Mutex};

use backend::CakeBackendModel;
use util::fps::Fps;

pub struct CakeViewModel {
    pub fps: Fps,
}

impl CakeViewModel {
    pub fn new() -> Self {
        CakeViewModel { fps: Fps::new() }
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
