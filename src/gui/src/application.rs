use std::sync::{Arc, RwLock};

use wgpu::{Adapter, Device, Instance, Queue};

pub trait ApplicationGraphics {
    fn adapter(&self) -> &Adapter;
    fn device(&self) -> &Arc<Device>;
    fn queue(&self) -> &Arc<Queue>;
    fn instance(&self) -> &Arc<Instance>;
}

pub trait Application<Model> {
    fn app_data(&self) -> &Arc<dyn ApplicationGraphics>;
    fn model(&self) -> &Arc<RwLock<Model>>;

    fn start(&self) {
        
    }
}
