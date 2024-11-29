pub trait Handler<T> {
    type R;

    fn call(self, it: impl Iterator<Item=T>) -> Self::R;
}

// Implementing `Access<T>` means that you somehow contain a bunch of `T`, to which you want to grant access.
pub trait Access<T> {
    fn access_mut<'a, H: Handler<&'a mut T>>(&'a mut self, h: H) -> H::R;
    fn access<'a, H: Handler<&'a T>>(&'a self, h: H) -> H::R;
    fn into_access<H: Handler<T>>(self, h: H) -> H::R;
}

// Example implementation:
impl<T> Access<T> for Vec<T> {
    fn access_mut<'a, H: Handler<&'a mut T>>(&'a mut self, h: H) -> H::R {
        h.call(self.iter_mut())
    }

    fn access<'a, H: Handler<&'a T>>(&'a self, h: H) -> H::R {
        h.call(self.iter())
    }

    fn into_access<H: Handler<T>>(self, h: H) -> H::R {
        h.call(self.into_iter())
    }
}
