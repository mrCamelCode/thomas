use std::any::Any;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Message<T> {
    pub typ: String,
    pub payload: T,
}
#[allow(dead_code)]
impl<T> Message<T>
where
    T: 'static,
{
    pub fn new(typ: &str, payload: T) -> Self {
        Self {
            typ: typ.to_string(),
            payload,
        }
    }

    pub fn get_payload<'a>(message: &'a Message<Box<dyn Any>>) -> Option<&'a T> {
        message.payload.downcast_ref::<T>()
    }
}
