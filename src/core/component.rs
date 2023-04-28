pub trait Component {
    fn name() -> &'static str
    where
        Self: Sized;
}
