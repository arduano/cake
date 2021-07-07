

#[macro_export]
macro_rules! palette {
    ( $col:ident ) => {
        gui::elements::FGetCol::<CakeModel>::new(|model| model.view.palette.$col)
    };
}