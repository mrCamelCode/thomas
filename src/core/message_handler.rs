pub struct Message<T> {
    typ: &str,
    payload: T,
}
pub trait MessageHandler<T> {
    fn handle(&mut self, message: Message<T>);
}
