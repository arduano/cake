#[macro_export]
macro_rules! return_option {
  ($a:expr) => {{
      match $a {
          Some(v) => return v,
          _ => panic!("Value already dropped!"),
      }
  }};
}