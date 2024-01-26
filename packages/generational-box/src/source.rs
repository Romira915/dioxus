use std::any::Any;

pub trait Source {
    fn read(&self) -> &dyn Any;
    fn write(&mut self) -> &mut dyn Any;
}

/// A source that does no mechanics on read/write
struct DefaultSource<T: 'static>(T);

impl<T: 'static> Source for DefaultSource<T> {
    #[inline]
    fn read(&self) -> &dyn Any {
        &self.0
    }

    #[inline]
    fn write(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}
