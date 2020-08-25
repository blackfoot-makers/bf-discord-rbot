use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};
use std::{thread, time};

#[derive(Clone)]
pub struct ThreadControl(Arc<AtomicBool>);

#[allow(dead_code)]
impl ThreadControl {
  pub fn new() -> Self {
    Self(Arc::new(AtomicBool::new(false)))
  }

  pub fn pause(&self) {
    self.0.store(true, Ordering::SeqCst);
  }

  pub fn resume(&self) {
    self.0.store(false, Ordering::SeqCst);
  }

  pub fn check(&self) {
    while self.0.load(Ordering::SeqCst) {
      thread::sleep(time::Duration::from_secs(1));
    }
  }
}
