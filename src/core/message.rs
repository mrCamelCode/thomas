use std::any::Any;

#[allow(dead_code)]
pub struct Message<T> {
    typ: String,
    payload: T,
}
#[allow(dead_code)]
impl<T> Message<T>
where
    T: 'static,
{
    fn get_payload<'a>(message: &'a Message<Box<dyn Any>>) -> Option<&'a T> {
        message.payload.downcast_ref::<T>()
    }
}
